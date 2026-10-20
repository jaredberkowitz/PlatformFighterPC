extends RefCounted
## Baked light for a stage's scenery (the stage refinement's grove pilot, docs/art/stages/BAKE_PILOT.md). Presentation only.
##
## The stage is built by code (`StageArt`), so its baked light is made in three steps:
## 1. `export_scene` writes the built scenery's toon-shaded meshes (and everything that casts occlusion on them) to a glTF file, each
##    mesh named by its place in the build order (`b0007`), the occluders `o0003`. Run with `--bake-export` on the stage demo.
## 2. Blender bakes ambient occlusion for every `b` mesh into its own texture over a new second UV set (art/blender/bake_stage.py) and
##    writes the meshes back with that UV set.
## 3. `apply` swaps each built mesh for its baked copy and turns on the toon shader's `use_baked_ao` with its texture.
## The bake belongs to one exact stage (its theme and block layout, `key`), so custom stages on the same theme are left as they are.

const FighterView := preload("res://scripts/fighter_view.gd")
const StageArt := preload("res://scripts/stage_art.gd")
const DIR := "res://art/baked/%s/"
## Modelled stage fronts, by the exact stage they were built on (`key`): art/blender/make_grove_front.py, the grove pilot's step 2.
const FRONTS := {"grove_14d06e65": "res://models/stages/grove_front.glb", "meadow_62409190": "res://models/stages/meadow_front.glb",
	"ocean_d9c5edff": "res://models/stages/beach_front.glb"}
## Modelled trees by theme (art/blender/make_grove_trees.py, option C, the owner's pick): placed where each code-built tree stood, so
## custom stages on the theme take them too.
const TREES := {"grove": "res://models/stages/grove_tree.glb"}
## Modelled background ground and bushes, by the exact stage they were built round ([ground, bush]; make_grove_ground.py, option A, the
## owner's pick): the ground stands on the side trees' footing for this layout.
const GROUNDS := {"grove_14d06e65": ["res://models/stages/grove_ground.glb", "res://models/stages/grove_bush.glb"]}
## The toon shaders whose materials take the baked occlusion (they share shaders/toon_common.gdshaderinc).
const BAKEABLE := ["res://shaders/toon.gdshader"]
## Shaders that cast occlusion on the baked meshes without being baked themselves.
const OCCLUDES := ["res://shaders/toon.gdshader", "res://shaders/toon_foliage.gdshader", "res://shaders/toon_cloth.gdshader"]


## The bake's folder name for a stage: its theme and a hash of its blocks and platforms.
static func key(theme_name: String, rects: Array) -> String:
	var h := 0
	for r in rects:
		for v in (r as PackedFloat32Array):
			h = hash([h, snappedf(v, 0.01)])
	return "%s_%08x" % [theme_name, h & 0xffffffff]


## A bake setting, `--bake-<name>=<value>` on the command line (for tuning captures) or its default.
static func _tune(name: String, default: float) -> float:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--bake-%s=" % name):
			return float(a.get_slice("=", 1))
	return default


static func _shader_path(m: Material) -> String:
	if m is ShaderMaterial and (m as ShaderMaterial).shader != null:
		return (m as ShaderMaterial).shader.resource_path
	return ""


## Whether every surface of `mi` is drawn by a bakeable toon material.
## Whether `n` or anything above it is about to be freed (a stage rebuilt in the same frame leaves the old one in the tree until then).
static func _dying(n: Node) -> bool:
	while n != null:
		if n.is_queued_for_deletion():
			return true
		n = n.get_parent()
	return false


static func _bakeable(mi: MeshInstance3D) -> bool:
	# (A modelled front carries its own baked occlusion: it casts occlusion on the rest, it is not baked again.)
	if mi.has_meta("prebaked"):
		return false
	if mi.mesh == null or not mi.is_visible_in_tree() or mi.mesh.get_surface_count() == 0 or _dying(mi):
		return false
	for s in mi.mesh.get_surface_count():
		if not BAKEABLE.has(_shader_path(mi.get_active_material(s))):
			return false
	return true


static func _occludes(gi: GeometryInstance3D) -> bool:
	if not gi.is_visible_in_tree() or _dying(gi):
		return false
	if gi is MeshInstance3D:
		var mi := gi as MeshInstance3D
		if mi.mesh == null or mi.mesh.get_surface_count() == 0:
			return false
		return OCCLUDES.has(_shader_path(mi.get_active_material(0)))
	if gi is MultiMeshInstance3D:
		var mm := (gi as MultiMeshInstance3D).multimesh
		if mm == null or mm.mesh == null:
			return false
		var m: Material = gi.material_override if gi.material_override != null else mm.mesh.surface_get_material(0)
		return OCCLUDES.has(_shader_path(m))
	return false


## The meshes to bake under `root`, in build order (the same every time: the build is seeded).
static func candidates(root: Node) -> Array:
	var out: Array = []
	for mi: MeshInstance3D in root.find_children("*", "MeshInstance3D", true, false):
		if _bakeable(mi):
			out.append(mi)
	return out


## Writes the built scenery under `root` to `<DIR>/scene_in.glb` for Blender, with a manifest. Returns how many meshes it wrote.
static func export_scene(root: Node3D, bake_key: String, rects: Array = [], front_z := 0.0) -> int:
	var dir := DIR % bake_key
	DirAccess.make_dir_recursive_absolute(ProjectSettings.globalize_path(dir))
	# (Godot's importer leaves the folder alone: the files are read at run time as they are.)
	FileAccess.open(dir + ".gdignore", FileAccess.WRITE).store_string("")
	var out := Node3D.new()
	out.name = "Bake"
	# (The stage's collision as guides for authoring in Blender: each block or platform [left, right, top, bottom, platform], and the
	# plane the fronts stand on.)
	var guides: Array = []
	for r in rects:
		guides.append(Array(r as PackedFloat32Array))
	var manifest := {"key": bake_key, "rects": guides, "front_z": front_z, "meshes": []}
	var list := candidates(root)
	for i in list.size():
		var mi: MeshInstance3D = list[i]
		var copy := MeshInstance3D.new()
		copy.name = "b%04d" % i
		copy.mesh = mi.mesh
		copy.transform = mi.global_transform
		var counts: Array = []
		for s in mi.mesh.get_surface_count():
			# (One named plain material per surface, so Blender keeps the surfaces apart and `apply` can put them back in order.)
			var std := StandardMaterial3D.new()
			std.resource_name = "b%04d_%d" % [i, s]
			var c = (mi.get_active_material(s) as ShaderMaterial).get_shader_parameter("albedo")
			std.albedo_color = c if c is Color else Color(0.7, 0.7, 0.7)
			copy.set_surface_override_material(s, std)
			counts.append((mi.mesh.surface_get_arrays(s)[Mesh.ARRAY_VERTEX] as PackedVector3Array).size())
		out.add_child(copy)
		manifest.meshes.append({"name": copy.name, "path": str(root.get_path_to(mi)), "surfaces": counts,
			"aabb": [mi.mesh.get_aabb().size.x, mi.mesh.get_aabb().size.y, mi.mesh.get_aabb().size.z]})
	var k := 0
	for gi: GeometryInstance3D in root.find_children("*", "GeometryInstance3D", true, false):
		if list.has(gi) or not _occludes(gi):
			continue
		var copy: GeometryInstance3D
		if gi is MultiMeshInstance3D:
			copy = MultiMeshInstance3D.new()
			(copy as MultiMeshInstance3D).multimesh = (gi as MultiMeshInstance3D).multimesh
		else:
			copy = MeshInstance3D.new()
			(copy as MeshInstance3D).mesh = (gi as MeshInstance3D).mesh
		copy.name = "o%04d" % k
		copy.transform = gi.global_transform
		out.add_child(copy)
		k += 1
	var doc := GLTFDocument.new()
	var state := GLTFState.new()
	var err := doc.append_from_scene(out, state)
	if err == OK:
		err = doc.write_to_filesystem(state, ProjectSettings.globalize_path(dir + "scene_in.glb"))
	out.free()
	FileAccess.open(dir + "manifest.json", FileAccess.WRITE).store_string(JSON.stringify(manifest, "\t"))
	print("stage bake: exported %d meshes and %d occluders to %s (%s)" % [list.size(), k, dir, error_string(err)])
	return list.size()


## Swaps the built meshes under `root` for their baked copies, if this stage has a bake. Returns how many it swapped.
static func apply(root: Node3D, bake_key: String) -> int:
	var dir := DIR % bake_key
	if OS.get_cmdline_user_args().has("--no-bake") or not FileAccess.file_exists(dir + "scene_baked.glb"):
		return 0
	var manifest = JSON.parse_string(FileAccess.get_file_as_string(dir + "manifest.json"))
	var doc := GLTFDocument.new()
	var state := GLTFState.new()
	if not (manifest is Dictionary) or doc.append_from_file(ProjectSettings.globalize_path(dir + "scene_baked.glb"), state) != OK:
		push_warning("stage bake %s could not be read" % bake_key)
		return 0
	var scene := doc.generate_scene(state)
	var baked := {}
	for mi: MeshInstance3D in scene.find_children("*", "MeshInstance3D", true, false):
		baked[String(mi.name)] = mi.mesh
	var list := candidates(root)
	var swapped := 0
	for i in list.size():
		var mi: MeshInstance3D = list[i]
		var nm := "b%04d" % i
		if i >= (manifest.meshes as Array).size() or not baked.has(nm) or not FileAccess.file_exists(dir + nm + ".png"):
			continue
		var entry: Dictionary = manifest.meshes[i]
		# (The same mesh as when it was baked, or nothing: a changed build must be baked again, not patched with the wrong light.)
		var size := mi.mesh.get_aabb().size
		if (entry.surfaces as Array).size() != mi.mesh.get_surface_count() or Vector3(entry.aabb[0], entry.aabb[1], entry.aabb[2]).distance_to(size) > 0.01:
			continue
		var src: Mesh = baked[nm]
		# (And the baked copy itself the same size: a bake made before the scenery changed, under the same names, swapped other meshes in.)
		if src.get_aabb().size.distance_to(size) > 0.02:
			continue
		# The baked surfaces back in the built mesh's order, by their material names (`b0007_2` is surface 2).
		var order := {}
		for s in src.get_surface_count():
			var m := src.surface_get_material(s)
			var parts := (m.resource_name if m != null else "").split("_")
			if parts.size() == 2:
				order[int(parts[1])] = s
		if order.size() != mi.mesh.get_surface_count():
			continue
		var mesh := ArrayMesh.new()
		for s in mi.mesh.get_surface_count():
			mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, src.surface_get_arrays(order[s]))
		var img := Image.load_from_file(ProjectSettings.globalize_path(dir + nm + ".png"))
		if img == null:
			continue
		img.generate_mipmaps()
		var tex := ImageTexture.create_from_image(img)
		# (Full near the fight, fading with depth: baked into the far ranges, the folds' crevices gave the distance more contrast than the
		# style sheet allows; the distance keeps its clear air.)
		var depth := -(mi.global_transform * mi.mesh.get_aabb().get_center()).z
		var strength := 1.0 - 0.7 * smoothstep(15.0, 60.0, depth)
		var mats: Array = []
		for s in mi.mesh.get_surface_count():
			var m := (mi.get_active_material(s) as ShaderMaterial).duplicate() as ShaderMaterial
			m.set_shader_parameter("use_baked_ao", true)
			m.set_shader_parameter("baked_ao", tex)
			m.set_shader_parameter("baked_ao_light", _tune("light", 0.85))
			m.set_shader_parameter("baked_ao_curve", _tune("curve", 1.8))
			m.set_shader_parameter("baked_ao_strength", strength)
			mats.append(m)
		mi.material_override = null
		mi.mesh = mesh
		for s in mats.size():
			mi.set_surface_override_material(s, mats[s])
		swapped += 1
	scene.free()
	print("stage bake: %d of %d meshes take baked light (%s)" % [swapped, list.size(), bake_key])
	return swapped


## The modelled front for this exact stage, or "" (none, or `--no-front` on the command line for comparisons).
static func front_path(bake_key: String) -> String:
	if OS.get_cmdline_user_args().has("--no-front") or front_mock_path() != "":
		return ""
	var path: String = FRONTS.get(bake_key, "")
	return path if path != "" and ResourceLoader.exists(path) else ""


static func front_mock_path() -> String:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--front-mock="):
			return a.substr(13)
	return ""


## A modelled stage front to try (`--front-mock=<glb>`, the grove pilot's step 2 mocks, art/blender/make_grove_front.py): the code-built
## island (`quilted_island`, `Underside`) hidden and the model placed in its stead, each material family painted from the stage's
## palette with the scenery's toon material, falling into shade down the front as the code-built fronts do. Returns whether it placed one.
static func mock_front(root: Node3D, t: Dictionary) -> bool:
	var path := front_mock_path()
	if path == "" or not FileAccess.file_exists(path):
		return false
	var doc := GLTFDocument.new()
	var state := GLTFState.new()
	if doc.append_from_file(path, state) != OK:
		push_warning("front mock %s could not be read" % path)
		return false
	hide_code_front(root, t)
	place_front(root, t, doc.generate_scene(state))
	print("front mock: placed %s" % path.get_file())
	return true


## A colour from the stage's [backdrop] palette.
static func _pal_of(t: Dictionary, key: String, default: Color) -> Color:
	return StageArt.palette(String(t.get("name", ""))).get("backdrop", {}).get(key, default)


## Hides the code-built front a modelled one replaces: the quilt, the block's box and its faceted ledge, and a floating island's keel.
static func hide_code_front(root: Node3D, t: Dictionary) -> void:
	for n in root.find_children("*", "", true, false):
		if _dying(n) or not (n is Node3D):
			continue
		var nm := String(n.name)
		# (The keel only where the island floats: on a terrace stage `Underside` is the hill the stage stands on, the meadow's.)
		var keel: bool = nm.begins_with("Underside") and not t.get("terrace", false)
		# (`begins_with`: a rebuilt stage's nodes take a number, `block2`.)
		if nm.begins_with("quilted_island") or keel or nm.begins_with("block") or nm.begins_with("faceted_ledge") or nm.begins_with("strata_ledges") 				or n.has_meta("code_front"):
			(n as Node3D).visible = false


## Places a modelled front (`scene`, from make_grove_front.py) under `root`: each material family painted from the stage's palette with
## the scenery's toon material and the model's own baked occlusion (vertex colours), falling into shade down the front as the
## code-built fronts do.
static func place_front(root: Node3D, t: Dictionary, scene: Node3D) -> void:
	var quilt: Color = t.get("quilt", Color(0.88, 0.66, 0.42))
	var grass: Color = t.get("grass", Color(0.5, 0.8, 0.3))
	var colours := {"quilt": quilt, "button": quilt.darkened(0.35), "earth": (t.get("soil_dark", Color(0.5, 0.35, 0.2)) as Color).darkened(0.55),
		# (Greyed toward green and the roots a darker bark: under the grove's pink light grey stone read lavender and the bark pink.)
		"rock": Color(0.48, 0.55, 0.44), "root": Color(0.46, 0.37, 0.3), "moss": grass.darkened(0.12),
		"moss_shade": grass.darkened(0.5),
		# (The meadow's families, art/blender/make_meadow_front.py: its light-moss ledge, the farm's wood, warm stone, soil, the lawn.)
		"ledge": t.get("ledge", Color(0.4, 0.62, 0.32)), "log": (t.get("wood", Color(0.86, 0.62, 0.36)) as Color).darkened(0.12),
		"log_end": (t.get("wood", Color(0.86, 0.62, 0.36)) as Color).lightened(0.25), "stone": Color(0.66, 0.64, 0.6),
		"soil": t.get("soil", Color(0.76, 0.56, 0.36)), "grass": grass,
		# (The beach's families, art/blender/make_beach_front.py: the deck's honey for beams, the posts' brown, a pale waterline, iron,
		# rope, harbour stone, foam.)
		# (A step lighter than the old piles: under the deck's baked shade they read nearly black.)
		"post": _pal_of(t, "pile_wood", Color(0.44, 0.29, 0.19)).lightened(0.18), "beam": _pal_of(t, "deck_wood", Color(0.5, 0.32, 0.18)),
		"waterline": Color(0.72, 0.7, 0.58), "iron": Color(0.22, 0.2, 0.22), "rope": Color(0.78, 0.64, 0.42),
		# (The harbour town's own quay stone, warm: a neutral grey went cold blue-grey under the sea's sky light.)
		"harbour_stone": _pal_of(t, "quay", Color(0.7, 0.6, 0.48)), "foam": Color(0.9, 0.97, 1.0), "weed": Color(0.26, 0.46, 0.3),
		# (The canyon's strata, art/blender/make_canyon_front.py: its sandstone, its deep brick and a pale band, as the code-built strata.)
		# (Wider apart than the code-built strata's: under the dusk light three close reds read as one.)
		"strata_a": t.get("soil", Color(0.86, 0.5, 0.32)), "strata_b": (t.get("soil_dark", Color(0.68, 0.34, 0.24)) as Color).darkened(0.12),
		"strata_c": (t.get("soil", Color(0.86, 0.5, 0.32)) as Color).lerp(Color(1.0, 0.86, 0.66), 0.6)}
	scene.name = "ModelledFront"
	root.add_child(scene)
	var top := -INF
	for mi: MeshInstance3D in scene.find_children("*", "MeshInstance3D", true, false):
		top = maxf(top, (mi.global_transform * mi.mesh.get_aabb().end).y)
	for mi: MeshInstance3D in scene.find_children("*", "MeshInstance3D", true, false):
		for s in mi.mesh.get_surface_count():
			var src := mi.mesh.surface_get_material(s)
			var fam := src.resource_name if src != null else "rock"
			var m := (FighterView.toon(colours.get(fam, Color(0.6, 0.6, 0.6)), true) as ShaderMaterial).duplicate() as ShaderMaterial
			m.set_shader_parameter("facets", fam != "quilt")
			# (The model carries its own baked occlusion in its vertex colours.)
			m.set_shader_parameter("vertex_shade", true)
			m.set_shader_parameter("ramp_softness", 0.3 if fam == "quilt" else 0.1)
			# (Not the foam on the water: falling into the front's shade it went grey.)
			# (Nor on the beach's dock: its supports stand in open sea and air, not in a bank of earth; falling into the front's calm haze
			# the stone went flat grey and the posts blue.)
			m.set_shader_parameter("fall_by_height", fam != "foam" and String(t.get("name", "")) != "ocean")
			# (Gentler on posts standing in the sea: at the cliffs' 0.4 the beach's piles fell nearly black.)
			m.set_shader_parameter("front_fall", 0.15 if fam in ["post", "waterline", "weed", "rope"] else 0.4)
			m.set_shader_parameter("front_span", 7.0)
			mi.set_surface_override_material(s, m)
		mi.set_instance_shader_parameter("front_top", top)
		mi.set_meta("prebaked", true)


## The theme's modelled tree: a glTF path to try (`--tree-mock=<glb>`), the shipped model (`TREES`), or null (none, or `--no-trees`).
static func tree_source(theme_name: String) -> Variant:
	var mock := tree_mock_path()
	if mock != "":
		return mock
	if OS.get_cmdline_user_args().has("--no-trees") or not TREES.has(theme_name) or not ResourceLoader.exists(TREES[theme_name]):
		return null
	return load(TREES[theme_name])


static func tree_mock_path() -> String:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--tree-mock="):
			return a.substr(12)
	return ""


## Places the modelled tree (`source`: a PackedScene, or a glTF path for mocks) where each code-built tree stood (`StageArt.tree_slots`),
## at its size, painted from the stage's palette with the scenery's toon material and the model's own baked occlusion (vertex colours).
static func place_trees(root: Node3D, t: Dictionary, slots: Array, source: Variant) -> void:
	var doc := GLTFDocument.new()
	var state := GLTFState.new()
	if source is String and doc.append_from_file(source, state) != OK:
		push_warning("tree mock %s could not be read" % source)
		return
	var grass: Color = t.get("grass", Color(0.5, 0.8, 0.3))
	# (The bark the front's roots wear, so the great tree and its roots are one tree.)
	var bark := Color(0.46, 0.37, 0.3)
	for a in OS.get_cmdline_user_args():
		if a == "--tree-bark=light":
			bark = t.get("great_bark", StageArt.palette(String(t.get("name", ""))).get("backdrop", {}).get("great_bark", Color(0.66, 0.49, 0.38)))
	var colours := {"bark": bark, "leaf_lit": Color(0.46, 0.74, 0.36), "leaf_shade": Color(0.2, 0.46, 0.3),
		"moss": grass.darkened(0.12)}
	for slot in slots:
		var tree: Node3D = (source as PackedScene).instantiate() if source is PackedScene else doc.generate_scene(state)
		tree.name = "ModelledTree"
		tree.position = slot[0]
		tree.scale = Vector3.ONE * float(slot[1])
		root.add_child(tree)
		for mi: MeshInstance3D in tree.find_children("*", "MeshInstance3D", true, false):
			for s in mi.mesh.get_surface_count():
				var src := mi.mesh.surface_get_material(s)
				var fam := src.resource_name if src != null else "bark"
				var m := (FighterView.toon(colours.get(fam, Color(0.6, 0.6, 0.6)), fam != "bark") as ShaderMaterial).duplicate() as ShaderMaterial
				m.set_shader_parameter("facets", fam != "bark")
				m.set_shader_parameter("vertex_shade", true)
				m.set_shader_parameter("ramp_softness", 0.25 if fam == "bark" else 0.1)
				mi.set_surface_override_material(s, m)
			mi.set_meta("prebaked", true)
	if source is String:
		print("tree mock: placed %d trees from %s" % [slots.size(), (source as String).get_file()])


## A command-line value (`<prefix><value>`), or "".
static func flag(prefix: String) -> String:
	for a in OS.get_cmdline_user_args():
		if a.begins_with(prefix):
			return a.substr(prefix.length())
	return ""


## The colour of each modelled background family, from the stage's palette.
static func _ground_colours(t: Dictionary) -> Dictionary:
	var bd: Dictionary = StageArt.palette(String(t.get("name", ""))).get("backdrop", {})
	var land: Color = bd.get("fore_band", Color(0.26, 0.54, 0.38))
	# (The ledges' tops in the island lawn's yellow-green, a step deeper, and the far ones a cooler green, against deep-teal cliffs: in
	# the rise's own teal, top and cliff ran together and the ledges read as one mass.)
	land = Color(0.44, 0.7, 0.3)
	return {"grass": land, "grass_far": Color(0.38, 0.62, 0.36), "cliff": Color(0.16, 0.36, 0.32),
		"earth": Color(0.24, 0.2, 0.18), "rock": Color(0.48, 0.55, 0.44), "root": Color(0.46, 0.37, 0.3), "bark": Color(0.46, 0.37, 0.3),
		"leaf_lit": Color(0.46, 0.74, 0.36), "leaf_shade": Color(0.2, 0.46, 0.3), "moss": (t.get("grass", Color(0.5, 0.8, 0.3)) as Color).darkened(0.12),
		"fern": Color(0.36, 0.66, 0.34), "fern_shade": Color(0.18, 0.42, 0.26)}


## Paints a modelled background piece: each family's toon material, its own baked occlusion (vertex colours), and the air at its depth.
static func _paint_ground(node: Node3D, colours: Dictionary) -> void:
	for mi: MeshInstance3D in node.find_children("*", "MeshInstance3D", true, false):
		for s in mi.mesh.get_surface_count():
			var src := mi.mesh.surface_get_material(s)
			var fam := src.resource_name if src != null else "grass"
			var m := (FighterView.toon(colours.get(fam, Color(0.6, 0.6, 0.6)), false) as ShaderMaterial).duplicate() as ShaderMaterial
			m.set_shader_parameter("facets", fam != "bark")
			m.set_shader_parameter("vertex_shade", true)
			m.set_shader_parameter("ramp_softness", 0.12)
			mi.set_surface_override_material(s, m)
		mi.set_meta("prebaked", true)
		var depth := -(mi.global_transform * mi.mesh.get_aabb().get_center()).z
		StageArt._air(mi, depth)


## The modelled ground and bush for this exact stage: glTF paths to try (`--ground-mock=`, `--bush-mock=`), the shipped models
## (`GROUNDS`, as PackedScenes), or [] (none, or `--no-ground`).
static func ground_source(bake_key: String) -> Array:
	if flag("--ground-mock=") != "":
		return [flag("--ground-mock="), flag("--bush-mock=")]
	if OS.get_cmdline_user_args().has("--no-ground") or not GROUNDS.has(bake_key):
		return []
	var paths: Array = GROUNDS[bake_key]
	if not ResourceLoader.exists(paths[0]) or not ResourceLoader.exists(paths[1]):
		return []
	return [load(paths[0]), load(paths[1])]


## A scene from `source`: a PackedScene, or a glTF path (mocks); null without one.
static func _scene_from(source: Variant) -> Node3D:
	if source is PackedScene:
		return (source as PackedScene).instantiate() as Node3D
	if source is String and source != "":
		var doc := GLTFDocument.new()
		var state := GLTFState.new()
		if doc.append_from_file(source, state) == OK:
			return doc.generate_scene(state)
	return null


## Places the modelled ground (`source[0]`, built in world space; art/blender/make_grove_ground.py) and the modelled bush (`source[1]`)
## at each of the code-built bushes' places (`StageArt.bush_slots`), painted by `_paint_ground`.
static func place_ground(root: Node3D, t: Dictionary, source: Array) -> void:
	var colours := _ground_colours(t)
	var ground := _scene_from(source[0])
	if ground != null:
		ground.name = "ModelledGround"
		root.add_child(ground)
		_paint_ground(ground, colours)
	for slot in StageArt.bush_slots:
		var bush := _scene_from(source[1])
		if bush == null:
			break
		bush.position = slot[0]
		bush.scale = Vector3.ONE * float(slot[1])
		bush.rotation_degrees.y = float(slot[2])
		root.add_child(bush)
		_paint_ground(bush, colours)
