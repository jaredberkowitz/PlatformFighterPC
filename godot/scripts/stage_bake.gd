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
const Lighting := preload("res://scripts/lighting.gd")
const WorldMotion := preload("res://scripts/world_motion.gd")
const DIR := "res://art/baked/%s/"
## Modelled stage fronts, by the exact stage they were built on (`key`): art/blender/make_grove_front.py, the grove pilot's step 2.
const FRONTS := {"grove_14d06e65": "res://models/stages/grove_front.glb", "meadow_62409190": "res://models/stages/meadow_front.glb",
	"ocean_d9c5edff": "res://models/stages/beach_front.glb", "sunset_19cad55b": "res://models/stages/canyon_front.glb",
	"city_2eadbb68": "res://models/stages/city_front.glb"}
## Modelled trees by theme (art/blender/make_grove_trees.py, option C, the owner's pick): placed where each code-built tree stood, so
## custom stages on the theme take them too.
# (The grove's in three shapes of the one tree, fidelity review 6, the owner's pick B: one tree at one size all round read as a clone.)
const TREES := {"grove": ["res://models/stages/grove_tree.glb", "res://models/stages/grove_tree_tall.glb",
	"res://models/stages/grove_tree_broad.glb"]}
## Modelled background ground and bushes, by the exact stage they were built round ([ground, bush]; make_grove_ground.py, option A, the
## owner's pick): the ground stands on the side trees' footing for this layout.
const GROUNDS := {
	# (The grove's floating ledges, docs/art/stages/BACKGROUNDS.md round 2: no bushes, which stood on the old ledge and would float.)
	"grove_14d06e65": ["res://models/stages/grove_paper.glb", ""],
	"meadow_62409190": ["res://models/stages/meadow_ground.glb", "res://models/stages/meadow_pines.glb"],
	# (The beach's land only: nothing stands on it in the bushes' place.)
	"ocean_d9c5edff": ["res://models/stages/beach_ground.glb", ""],
	"sunset_19cad55b": ["res://models/stages/canyon_paper.glb", ""]}
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
## The background's layout for Blender (art/blender/make_meadow_ground.py): the hill rows' and the rise's bumps ([centre x, half width,
## height, base y]) and where the pine clusters stand ([x, y, z, size, turn]), as `ground.json` beside the bake manifest.
static func export_ground(bake_key: String) -> void:
	var pines: Array = []
	for slot in StageArt.pine_slots:
		var at: Vector3 = slot[0]
		pines.append([at.x, at.y, at.z, slot[1], slot[2]])
	var isles: Array = []
	for i in StageArt.island_specs:
		var c: Vector3 = i[0]
		var a: Vector3 = i[1]
		var b: Vector3 = i[2]
		isles.append([c.x, c.y, c.z, a.x, a.y, a.z, b.x, b.y, b.z])
	var heads: Array = []
	for h in StageArt.headland_specs:
		var c: Vector3 = h[1]
		var r: Vector3 = h[2]
		heads.append([h[0], c.x, c.y, c.z, r.x, r.y, r.z])
	# Each ridge's crest: the highest point of its mesh in 4-unit bins across x (art/blender/make_grove_paper.py's cut-out flats).
	var ranges: Array = []
	for rn in StageArt.ridge_nodes:
		var mi := rn[0] as MeshInstance3D
		if not is_instance_valid(mi) or mi.mesh == null:
			continue
		var bins := {}
		var xf := mi.global_transform
		for v in mi.mesh.get_faces():
			var w := xf * v
			var b := floori(w.x / 4.0)
			bins[b] = maxf(float(bins.get(b, -INF)), w.y)
		var keys := bins.keys()
		keys.sort()
		var top: Array = []
		for b in keys:
			top.append([float(b) * 4.0 + 2.0, bins[b]])
		ranges.append({"z": (xf * mi.mesh.get_aabb()).get_center().z, "top": top})
	var mesas: Array = []
	for m in StageArt.mesa_specs:
		var b: AABB = m[0]
		mesas.append([b.position.x, b.position.y, b.position.z, b.end.x, b.end.y, b.end.z, 1 if m[1] else 0])
	var arch: Array = []
	for a in StageArt.arch_spec:
		var b: AABB = a
		arch = [b.position.x, b.position.y, b.position.z, b.end.x, b.end.y, b.end.z]
	var doc := {"mesas": mesas, "arch": arch, "ranges": ranges, "islands": isles, "headland": heads, "hills_near": StageArt._hill_bumps[0], "hills_far": StageArt._hill_bumps[1], "fore": StageArt._fore_bumps, "pines": pines,
		"farm": StageArt.farm_span, "hill": StageArt.hill_spec}
	FileAccess.open((DIR % bake_key) + "ground.json", FileAccess.WRITE).store_string(JSON.stringify(doc))


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
		# (Not the modelled front: the front builders bake a new one in its place, and inside the old one every face baked black,
		# fidelity review 6. By its mark, not its name: beside a first build still being freed it is `@Node3D@2`.)
		var in_front := false
		var n: Node = gi
		while n != null and n != root and not in_front:
			in_front = n.has_meta("modelled_front")
			n = n.get_parent()
		if in_front:
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


## Writes everything that casts light on the modelled pieces to `<DIR>/light_in.glb` for art/blender/light_stage.py (`--light-export`
## on the stage demo): every visible solid piece of the built stage in world space, without the modelled front, play area, ground and
## mood light themselves (marked `lit_piece`: Blender reads those from their own files; not by name, as a second build beside a first still
## being freed gets `@Node3D@2`). Returns how many it wrote.
static func export_light(root: Node3D, bake_key: String) -> int:
	var dir := DIR % bake_key
	DirAccess.make_dir_recursive_absolute(ProjectSettings.globalize_path(dir))
	FileAccess.open(dir + ".gdignore", FileAccess.WRITE).store_string("")
	var out := Node3D.new()
	out.name = "Light"
	var k := 0
	for gi: GeometryInstance3D in root.find_children("*", "GeometryInstance3D", true, false):
		if not _occludes(gi):
			continue
		var skip := false
		var n: Node = gi
		while n != null and n != root and not skip:
			skip = n.has_meta("lit_piece")
			n = n.get_parent()
		if skip:
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
		err = doc.write_to_filesystem(state, ProjectSettings.globalize_path(dir + "light_in.glb"))
	out.free()
	print("stage light: exported %d occluders to %s (%s)" % [k, dir, error_string(err)])
	return k


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
		"rock": Color(0.48, 0.55, 0.44), "root": _root_colour(t), "moss": grass.darkened(0.12),
		"moss_shade": grass.darkened(0.5),
		# (The meadow's families, art/blender/make_meadow_front.py: its light-moss ledge, the farm's wood, warm stone, soil, the lawn.)
		"ledge": t.get("ledge", Color(0.4, 0.62, 0.32)), "log": (t.get("wood", Color(0.86, 0.62, 0.36)) as Color).darkened(0.12),
		"log_end": (t.get("wood", Color(0.86, 0.62, 0.36)) as Color).lightened(0.25), "stone": Color(0.66, 0.64, 0.6),
		# (The meadow's fieldstone, fidelity review 8: a warm sandstone grey and a deep olive moss, so the front's calm and cool shade leave
		# a stone colour and the moss no longer reads neon.)
		"fieldstone": Color(0.8, 0.7, 0.56), "wall_moss": Color(0.36, 0.5, 0.26),
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
		"strata_a": _front_lift(t, t.get("soil", Color(0.86, 0.5, 0.32))),
		"strata_b": _front_lift(t, (t.get("soil_dark", Color(0.68, 0.34, 0.24)) as Color).darkened(0.12)),
		"strata_c": _front_lift(t, (t.get("soil", Color(0.86, 0.5, 0.32)) as Color).lerp(Color(1.0, 0.86, 0.66), 0.6)),
		# (The city's, art/blender/make_city_front.py: the code-built facade's warm stone, gold frames, curtains, neon and fittings.)
		"trim": Color(0.9, 0.8, 0.68), "frame": Color(0.86, 0.7, 0.46), "curtain_a": Color(0.92, 0.54, 0.58), "curtain_b": Color(0.56, 0.72, 0.92),
		"curtain_c": Color(0.92, 0.76, 0.44), "neon": Color(0.88, 0.3, 0.38), "pipe": Color(0.48, 0.48, 0.58), "pot": Color(0.78, 0.4, 0.3),
		"leaf": Color(0.36, 0.62, 0.3), "ac": Color(0.74, 0.75, 0.8), "spandrel": (t.get("soil_dark", Color(0.4, 0.3, 0.4)) as Color).darkened(0.3)}
	# (Lit rooms, dark panes and bulbs keep their own colour whatever the light, as the code-built facade's glass does.)
	var glow := {"glass_lit": Color(1.0, 0.72, 0.4), "glass_dark": Color(0.12, 0.13, 0.26), "bulb": Color(1.0, 0.94, 0.8)}
	scene.name = "ModelledFront"
	scene.set_meta("lit_piece", true)
	scene.set_meta("modelled_front", true)
	root.add_child(scene)
	var top := -INF
	for mi: MeshInstance3D in scene.find_children("*", "MeshInstance3D", true, false):
		top = maxf(top, (mi.global_transform * mi.mesh.get_aabb().end).y)
	for mi: MeshInstance3D in scene.find_children("*", "MeshInstance3D", true, false):
		for s in mi.mesh.get_surface_count():
			var src := mi.mesh.surface_get_material(s)
			var fam := src.resource_name if src != null else "rock"
			if glow.has(fam):
				var g := StandardMaterial3D.new()
				g.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
				g.albedo_color = glow[fam]
				g.vertex_color_use_as_albedo = true
				mi.set_surface_override_material(s, g)
				continue
			# (The city's wall in its own brick, laid in world space as the code-built facade's.)
			var m := ((StageArt._soil_material(t) if fam.begins_with("brick") else FighterView.toon(colours.get(fam, Color(0.6, 0.6, 0.6)), true)) \
				as ShaderMaterial).duplicate() as ShaderMaterial
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
			_paint_drift(m, t, false)
			mi.set_surface_override_material(s, m)
		mi.set_instance_shader_parameter("front_top", top)
		mi.set_meta("prebaked", true)


## Painted variation on a modelled piece's material (fidelity review 10; toon_common `paint_drift`): the palette's [backdrop]
## `paint_drift` [value patches, warm-cool wander, size, tops warm and walls cool], or `--paint-drift=A|B|C` to try (C as A on the play
## area only: on the front, a wall of small stones and logs that already vary, it only darkened and greyed); off without. `play` for the
## play area's pieces; the palette's optional fifth value 1 keeps it to them.
static func _paint_drift(m: ShaderMaterial, t: Dictionary, play: bool) -> void:
	var d: Array = (StageArt.palette(String(t.get("name", ""))).get("backdrop", {}) as Dictionary).get("paint_drift", [])
	var f := flag("--paint-drift=")
	if f != "":
		d = {"A": [1.3, 0.8, 14.0, 0.5], "B": [2.0, 1.2, 18.0, 0.7], "C": [1.3, 0.8, 14.0, 0.5, 1]}.get(f.to_upper(), d)
	if d.size() == 5 and int(d[4]) == 1 and not play:
		return
	if d.size() >= 4:
		m.set_shader_parameter("paint_drift", Vector4(d[0], d[1], d[2], d[3]))


## A modelled play area to try (`--play-mock=<glb>`, art/blender/make_meadow_play.py, the paper pilot): the stage's top and its
## platforms as paper card, placed in world space, each family painted from the palette with the scenery's toon material and the model's
## own baked occlusion.
static func place_play(root: Node3D, t: Dictionary, path: String) -> void:
	var scene := _scene_from(load(path) if path.begins_with("res://") else path)
	if scene == null:
		push_warning("play mock %s could not be read" % path)
		return
	var grass: Color = t.get("grass", Color(0.5, 0.8, 0.3))
	var wood: Color = t.get("wood", Color(0.86, 0.62, 0.36))
	var path_c: Color = t.get("path_colour", Color(0.94, 0.9, 0.74))
	# (The grove's path a worn earth, fidelity review 14 item 8: its pale cream went lavender under the pink light, a flat stripe.
	# `--path=A|B`; shipping writes it into grove.cfg's `path_colour`.)
	if String(t.get("name", "")) == "grove":
		path_c = {"A": Color(0.78, 0.64, 0.5), "B": Color(0.7, 0.55, 0.4)}.get(flag("--path=").to_upper(), path_c)
	var colours := {"lawn": grass, "lawn_under": grass.darkened(0.3), "path": path_c, "card": grass.lerp(Color(0.97, 0.96, 0.86), 0.45),
		"soil": t.get("soil", Color(0.76, 0.56, 0.36)), "stone": Color(0.66, 0.64, 0.6), "fringe": grass.lightened(0.08),
		"plank": wood, "plank_b": wood.darkened(0.1), "plank_card": wood.lerp(Color(0.98, 0.94, 0.84), 0.45), "rope": Color(0.78, 0.64, 0.42),
		"shadow": Color(0.1, 0.12, 0.08)}
	# (The other stages' families, art/blender/make_play.py: each stage's top, the layers under it, its cut fringe and its platforms.)
	var cream := Color(0.97, 0.96, 0.86)
	var soil: Color = t.get("soil", Color(0.76, 0.56, 0.36))
	var soil_dark: Color = t.get("soil_dark", Color(0.5, 0.35, 0.25))
	var top: Color = grass
	var layer1: Color = grass.darkened(0.25)
	var layer2: Color = soil_dark
	var fringe: Color = grass.lightened(0.08)
	match String(t.get("name", "")):
		"sunset":
			# (A step paler and cooler than the stage's sand, the readability pass: the orange fighter stood on sand of its own hue,
			# apart by value alone.)
			top = grass.lerp(Color(0.98, 0.93, 0.82), 0.35)
			layer1 = soil
			layer2 = soil_dark.darkened(0.12)
			fringe = soil
		"ocean":
			top = _pal_of(t, "deck_wood", Color(0.62, 0.42, 0.26))
			layer1 = top.darkened(0.2)
			layer2 = _pal_of(t, "pile_wood", Color(0.44, 0.29, 0.19)).lightened(0.18)
			fringe = top.darkened(0.1)
		"city":
			top = grass.lerp(Color(0.84, 0.56, 0.42), 0.9).lightened(0.18)
			# (A warm lit roof against the cool night, fidelity review 14 item 4: the palette's [backdrop] `roof_warm`, or `--warm-cool=`.)
			var warm := float((StageArt.palette("city").get("backdrop", {}) as Dictionary).get("roof_warm", 0.0))
			if flag("--warm-cool=") != "":
				warm = 0.35
			top = top.lerp(Color(0.95, 0.62, 0.4), warm)
			layer1 = grass.darkened(0.18)
			layer2 = soil_dark.darkened(0.35)
			fringe = (t.get("grass_light", top) as Color).lerp(Color(1.0, 0.9, 0.78), 0.6)
	# (The tops' wear, fidelity review 7: a deeper patch, a sunlit one and trodden earth, each from the top's own colour; on the dock
	# a weathered grey, wet wood and a replaced board; nail heads.)
	var base_top: Color = grass if String(t.get("name", "")) == "meadow" else top
	# (Steps of a fifth: at a tenth, under the print and the lawn's fall the lawn's contrast moved 0.071 to 0.073.)
	colours.merge({"top_deep": base_top.darkened(0.22), "top_lit": base_top.lightened(0.18).lerp(Color(1.0, 0.97, 0.82), 0.12),
		"top_worn": base_top.lerp(soil, 0.5), "nail": Color(0.24, 0.2, 0.2)}, true)
	if String(t.get("name", "")) == "ocean":
		colours.merge({"top_worn": base_top.lerp(Color(0.66, 0.62, 0.58), 0.4), "top_deep": base_top.darkened(0.18),
			"top_lit": base_top.lightened(0.14)}, true)
	var plate := wood.lerp(Color(0.3, 0.3, 0.42), 0.95)
	colours.merge({"top": top, "top_card": top.lerp(cream, 0.45), "layer1": layer1, "layer2": layer2, "fringe": fringe,
		"steel": plate, "steel_dark": (t.get("wood_dark", plate.darkened(0.3)) as Color).darkened(0.15), "steel_card": plate.lerp(cream, 0.35),
		# (The city's platforms in its own materials, fidelity review 8: plate seams, rivets and clamps, a safety paint, red-oxide primer, hazard stripes and
		# scaffold boards.)
		"steel_seam": plate.darkened(0.4), "rivet": plate.lerp(cream, 0.5), "steel_paint": Color(0.94, 0.7, 0.26),
		"steel_primer": Color(0.76, 0.36, 0.3),
		"hazard": Color(0.16, 0.14, 0.2), "deck_board": Color(0.74, 0.58, 0.44)}, true)
	# (The fighters' ground a step deeper, fidelity review 14 item 2: on the meadow, grove and canyon the yellow and orange fighters stood
	# on a top as light as they are, 0.04 to 0.08 apart in value where the city and beach hold 0.15 and 0.24. The palette's [backdrop]
	# `fight_ground` [how much darker, how much less saturated] or `--ground=A|B`: the walked surfaces only, the cut lips and the card
	# edges stay pale.)
	var ground: Array = (StageArt.palette(String(t.get("name", ""))).get("backdrop", {}) as Dictionary).get("fight_ground", [])
	if String(t.get("name", "")) in ["meadow", "grove", "sunset"]:
		# (Deeper at full colour: a first try took a fifth of the saturation too, and the lawns went a grey olive.)
		# (C and D, fidelity review 15 item 1: under the colourful shade the meadow's land round the fighters lightened, and P1's gap fell
		# from 0.138 to 0.105 at B.)
		ground = {"A": [0.2, 0.0], "B": [0.32, 0.08], "C": [0.42, 0.08], "D": [0.5, 0.1]}.get(flag("--ground=").to_upper(), ground)
	var bright: Dictionary = FLOOR_TRY.get(flag("--floor=").to_upper(), {})
	if ground.size() == 2 and bright.has("ground"):
		ground = [float(ground[0]) * float(bright.ground), float(ground[1]) * float(bright.ground)]
	if ground.size() == 2:
		for fam in ["lawn", "top", "path", "top_deep", "top_lit", "top_worn"]:
			if colours.has(fam):
				var c: Color = colours[fam]
				colours[fam] = Color.from_hsv(c.h, c.s * (1.0 - float(ground[1])), c.v * (1.0 - float(ground[0])))
	scene.name = "ModelledPlay"
	scene.set_meta("lit_piece", true)
	root.add_child(scene)
	for mi: MeshInstance3D in scene.find_children("*", "MeshInstance3D", true, false):
		for s in mi.mesh.get_surface_count():
			var src := mi.mesh.surface_get_material(s)
			var fam := src.resource_name if src != null else "lawn"
			if fam == "shadow":
				var sh := StandardMaterial3D.new()
				sh.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
				sh.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
				sh.albedo_color = Color(0.08, 0.1, 0.06, 0.35)
				mi.set_surface_override_material(s, sh)
				continue
			var m := (FighterView.toon(colours.get(fam, Color(0.6, 0.6, 0.6)), false) as ShaderMaterial).duplicate() as ShaderMaterial
			# (No facets: every piece is flat, and on the lawn's top the normal rebuilt from screen derivatives turned over, so it lit as an
			# underside and the fighters' contact shadows, which only fall on what faces up, vanished.)
			m.set_shader_parameter("facets", false)
			m.set_shader_parameter("vertex_shade", true)
			m.set_shader_parameter("ramp_softness", 0.12)
			# (Form on the ground, the light pass, the owner's pick 2026-10-21: the top falls softly into shade toward its front edge; the
			# lawns read flat once the scenery's ink went.)
			if fam in ["lawn", "top", "path", "fringe", "top_deep", "top_lit", "top_worn", "nail"]:
				m.set_shader_parameter("lawn_fall", 0.9 * float(bright.get("fall", 1.0)))
				m.set_shader_parameter("lawn_front", StageArt.STAGE_FRONT)
				m.set_shader_parameter("lawn_span", 6.0)
			# (Bright floors on trial, the owner 2026-10-29; see FLOOR_TRY.)
			if fam in ["lawn", "top", "path", "fringe", "top_deep", "top_lit", "top_worn", "plank", "plank_b", "deck_board", "steel"]:
				if bright.has("clouds"):
					m.set_shader_parameter("cloud_shade", float(bright.clouds))
				if bright.has("cast"):
					m.set_shader_parameter("shadow_amount", float(bright.cast))
				if bright.get("bare", false):
					m.set_shader_parameter("vertex_shade", false)
			_paint_drift(m, t, true)
			mi.set_surface_override_material(s, m)
		mi.set_meta("prebaked", true)
	print("play mock: placed %s" % path.get_file())


## Bright floors on trial (the owner, 2026-10-29: "the shadows are beginning to drown out the floors of the stages"; the canyon's top
## read almost all shadow, dark and muddy, where the papercraft reference's floors are bright with small shadows under things).
## `--floor=A|B|C`, on the walked tops and the platforms:
##   A  the drifting cloud shadows of fidelity review 7 (0.9 deep over 60% of every upward face near the stage) off them, the land
##      round them as it is: most of what read as a floor in shadow;
##   B  A, the sun's cast shadows on them lighter (`shadow_amount` 0.35 of the scenery's 0.8), their fall into shade toward the front
##      edge halved, and the deeper fighters' ground of fidelity review 14 item 2 (`fight_ground`) halved;
##   C  B, and the contact shade baked into the play pieces off on them, the cast shadows lighter still (0.2).
## (A first try without the clouds, B and C here less A, barely moved the canyon's top: the clouds were the shadow.)
const FLOOR_TRY := {
	"A": {"clouds": 0.0},
	"B": {"clouds": 0.0, "cast": 0.35, "fall": 0.5, "ground": 0.5},
	"C": {"clouds": 0.0, "cast": 0.2, "fall": 0.3, "ground": 0.5, "bare": true},
}


## Modelled play areas (the stage's top and its platforms, art/blender/make_meadow_play.py), by the exact stage they were built on.
const PLAYS := {"meadow_62409190": "res://models/stages/meadow_play.glb", "grove_14d06e65": "res://models/stages/grove_play.glb",
	"sunset_19cad55b": "res://models/stages/canyon_play.glb", "ocean_d9c5edff": "res://models/stages/beach_play.glb",
	"city_2eadbb68": "res://models/stages/city_play.glb"}


## The play area to place: `--play-mock=<glb>`, else the shipped model for this exact stage, else "" (none, or `--no-play`).
static func play_path(bake_key: String) -> String:
	if flag("--play-mock=") != "":
		return flag("--play-mock=")
	if OS.get_cmdline_user_args().has("--no-play") or not PLAYS.has(bake_key) or not ResourceLoader.exists(PLAYS[bake_key]):
		return ""
	return PLAYS[bake_key]


## Mood light in paper (art/blender/make_mood.py, review 2's idea 5, the owner's picks): the city's four searchlight beams in pairs
## (B), the grove's lanterns under the platforms and along the strings with warm pools on the lawn (C), the beach's ripple rings with
## a near-side crescent round the dock's and the boardwalk's posts (B). By the exact stage they were built on.
const MOODS := {"city_2eadbb68": "res://models/stages/city_mood.glb", "grove_14d06e65": "res://models/stages/grove_mood.glb",
	"ocean_d9c5edff": "res://models/stages/beach_mood.glb"}


## The mood light to place: a glTF path to try (`--mood-mock=<glb>`), the shipped model for this exact stage (`MOODS`), or null (none,
## or `--no-mood`).
static func mood_source(bake_key: String) -> Variant:
	if flag("--mood-mock=") != "":
		return flag("--mood-mock=")
	if OS.get_cmdline_user_args().has("--no-mood") or not MOODS.has(bake_key) or not ResourceLoader.exists(MOODS[bake_key]):
		return null
	return load(MOODS[bake_key])


## The world-building pieces' colours by family (art/blender/make_world.py, docs/art/stages/WORLD.md).
const WORLD_COLOURS := {"hull": Color(0.32, 0.5, 0.62), "hull_stripe": Color(0.95, 0.92, 0.84), "deck": Color(0.78, 0.6, 0.42),
	"cabin": Color(0.96, 0.94, 0.88), "cabin_roof": Color(0.82, 0.32, 0.28), "wood_dark": Color(0.45, 0.32, 0.24), "sail": Color(0.95, 0.9, 0.8),
	"rope": Color(0.86, 0.74, 0.52), "net": Color(0.42, 0.56, 0.52), "fish_crate": Color(0.84, 0.66, 0.44),
	"awning_a": Color(0.92, 0.4, 0.38), "awning_b": Color(0.98, 0.94, 0.86),
	"stall": Color(0.76, 0.56, 0.38), "coat_a": Color(0.36, 0.48, 0.72), "coat_b": Color(0.82, 0.5, 0.32), "coat_c": Color(0.5, 0.62, 0.4),
	"skin": Color(0.96, 0.8, 0.66), "flag_a": Color(0.92, 0.36, 0.36), "flag_b": Color(1.0, 0.84, 0.36), "flag_c": Color(0.4, 0.66, 0.86),
	"smoke": Color(0.9, 0.88, 0.92), "timber": Color(0.62, 0.42, 0.3), "timber_dark": Color(0.44, 0.3, 0.24), "iron": Color(0.32, 0.3, 0.36),
	"engine": Color(0.22, 0.24, 0.3), "engine_cab": Color(0.7, 0.22, 0.22), "car_a": Color(0.78, 0.5, 0.3), "car_b": Color(0.46, 0.58, 0.5),
	"tank": Color(0.62, 0.4, 0.3), "tank_dark": Color(0.42, 0.28, 0.24), "trail": Color(0.96, 0.84, 0.68), "canvas": Color(0.96, 0.93, 0.84),
	"blade": Color(0.86, 0.84, 0.8),
	"adobe": Color(0.9, 0.66, 0.5), "adobe_dark": Color(0.74, 0.5, 0.38), "window": Color(1.0, 0.82, 0.5),
	"straw": Color(0.98, 0.84, 0.46), "straw_end": Color(0.84, 0.66, 0.32), "gate_red": Color(0.86, 0.3, 0.26),
	"gate_dark": Color(0.3, 0.22, 0.24), "rock": Color(0.6, 0.56, 0.62), "turf": Color(0.48, 0.74, 0.36), "bark": Color(0.55, 0.38, 0.26),
	"bus": Color(0.95, 0.72, 0.28), "board_frame": Color(0.3, 0.28, 0.4), "steel": Color(0.4, 0.4, 0.52), "train": Color(0.36, 0.5, 0.7),
	"crown": Color(0.22, 0.42, 0.32), "firefly": Color(0.95, 1.0, 0.62),
	# (The quality pass's pieces, 2026-10-21: the paper dolls' trousers and hair, the train's brass bands and dark iron, the boxcars'
	# seams and roofs, the billboard's poster in a muted night palette, the crowns' lit tiers, the bus's stripe.)
	"trousers": Color(0.28, 0.3, 0.42), "hair": Color(0.3, 0.2, 0.16),
	"shoe": Color(0.22, 0.2, 0.26), "eye": Color(0.12, 0.1, 0.16), "white": Color(0.98, 0.96, 0.92), "engine_band": Color(0.86, 0.66, 0.34),
	"engine_dark": Color(0.16, 0.17, 0.22), "car_seam": Color(0.42, 0.28, 0.22), "car_roof": Color(0.3, 0.27, 0.3),
	"poster": Color(0.96, 0.9, 0.78), "poster_a": Color(0.95, 0.55, 0.45), "poster_b": Color(0.38, 0.66, 0.7), "poster_c": Color(0.3, 0.3, 0.48),
	"board_back": Color(0.22, 0.2, 0.3), "crown_lit": Color(0.3, 0.52, 0.4), "bus_stripe": Color(0.98, 0.94, 0.86)}
## World-building families drawn in their own light: lit windows, fireflies, the billboard's lit poster.
const WORLD_LIT := ["window", "firefly", "poster", "poster_a", "poster_b", "poster_c"]


## World-building (art/blender/make_world.py, docs/art/stages/WORLD.md, the owner's picks 2026-10-21): the beach's working pier and
## harbour life (a fishing boat at its buoy, nets and crates, a rowboat), the canyon's railroad (a trestle and train, a water tower), the
## meadow's fields at work and village life (haystacks, chimney smoke, a washing line, a scarecrow), the grove's far tree village and
## festival (great trees with lit huts and rope bridges, gates, streamers, fireflies), the city's billboard, rooftop party and street
## traffic. No animals (the owner). By the exact stage they were built on.
const WORLDS := {"ocean_d9c5edff": "res://models/stages/beach_world.glb", "sunset_19cad55b": "res://models/stages/canyon_world.glb",
	"meadow_62409190": "res://models/stages/meadow_world.glb", "grove_14d06e65": "res://models/stages/grove_world.glb",
	"city_2eadbb68": "res://models/stages/city_world.glb"}


## The world-building to place: a glTF path to try (`--world-mock=<glb>`), the shipped model for this exact stage (`WORLDS`), or null
## (none, or `--no-world`).
static func world_source(bake_key: String) -> Variant:
	if flag("--world-mock=") != "":
		return flag("--world-mock=")
	if OS.get_cmdline_user_args().has("--no-world") or not WORLDS.has(bake_key) or not ResourceLoader.exists(WORLDS[bake_key]):
		return null
	return load(WORLDS[bake_key])


## Places the world-building pieces (`source`: a PackedScene, or a glTF path for mocks): painted by family in the scenery's toon
## material with their own occlusion; the windows lit.
static func place_world(root: Node3D, source: Variant) -> void:
	var scene := _scene_from(source)
	if scene == null:
		push_warning("world-building %s could not be read" % str(source))
		return
	scene.name = "World"
	# (Printed and finished as the paper props are, `surfaces`.)
	scene.set_meta("paper_prop", true)
	root.add_child(scene)
	for mi: MeshInstance3D in scene.find_children("*", "MeshInstance3D", true, false):
		var src := mi.mesh.surface_get_material(0)
		var fam := (src.resource_name if src != null else "").get_slice(".", 0)
		var colour: Color = WORLD_COLOURS.get(fam, Color(0.7, 0.7, 0.7))
		if WORLD_LIT.has(fam):
			var lit := StandardMaterial3D.new()
			lit.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
			lit.albedo_color = colour
			mi.material_override = lit
			# (A firefly glows, as the lamps do: a speck alone was lost among the lanterns.)
			if fam == "firefly":
				StageArt._halo(mi, Vector3.ZERO, colour, 1.4)
			continue
		# (Through the stage's air at its depth, as the rest of the scenery: flat colours read pasted on far out. Lights cut through it.)
		var xf := Transform3D.IDENTITY
		var up: Node = mi
		while up != null and up != root:
			xf = (up as Node3D).transform * xf
			up = up.get_parent()
		colour = StageArt._air_colour(colour, (xf * mi.get_aabb().get_center()).z)
		var m := FighterView.toon(colour, true).duplicate() as ShaderMaterial
		m.set_shader_parameter("vertex_shade", true)
		mi.material_override = m
	# Its life (scripts/world_motion.gd): `--world-motion=off|place|all`.
	var mode := flag("--world-motion=")
	if mode != "off":
		var motion := WorldMotion.new()
		motion.name = "WorldMotion"
		motion.setup(scene, mode)
		scene.add_child(motion)
	print("world-building: placed %s" % (source.get_file() if source is String else (source as PackedScene).resource_path.get_file()))


## Places the mood light (`source`: a PackedScene, or a glTF path for mocks): searchlight beams, paper lanterns, ripple rings, each
## piece unshaded with its own brightness and fade in its vertex colours, so it reads as light whatever the scene's.
static func place_mood(root: Node3D, source: Variant) -> void:
	var scene := _scene_from(source)
	if scene == null:
		push_warning("mood light %s could not be read" % str(source))
		return
	# [colour, additive, double-sided, lit like the scenery]
	var looks := {"beam": [Color(1.0, 0.94, 0.86), true, true, false], "beam_card": [Color(0.86, 0.82, 0.95), false, true, false],
		"beam_edge": [Color(0.7, 0.62, 0.86), false, true, false],
		"glow": [Color(1.0, 0.76, 0.44), false, true, false], "halo": [Color(1.0, 0.72, 0.38), true, true, false],
		"pool": [Color(1.0, 0.78, 0.48), true, true, false], "ring": [Color(0.95, 0.98, 1.0), true, true, false],
		"spark": [Color(1.0, 1.0, 0.96), true, true, false], "cap": [Color(0.32, 0.16, 0.14), false, true, true],
		"cord": [Color(0.2, 0.17, 0.22), false, true, true]}
	scene.name = "ModelledMood"
	scene.set_meta("lit_piece", true)
	root.add_child(scene)
	for mi: MeshInstance3D in scene.find_children("*", "MeshInstance3D", true, false):
		mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		for s in mi.mesh.get_surface_count():
			var src := mi.mesh.surface_get_material(s)
			var look: Array = looks.get(src.resource_name if src != null else "glow", looks["glow"])
			if look[3]:
				mi.set_surface_override_material(s, FighterView.toon(look[0], false))
				continue
			var m := StandardMaterial3D.new()
			m.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
			m.vertex_color_use_as_albedo = true
			m.albedo_color = look[0]
			m.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
			if look[1]:
				m.blend_mode = BaseMaterial3D.BLEND_MODE_ADD
			if look[2]:
				m.cull_mode = BaseMaterial3D.CULL_DISABLED
			m.disable_receive_shadows = true
			mi.set_surface_override_material(s, m)
		mi.set_meta("prebaked", true)
	print("mood light: placed %s" % (source.get_file() if source is String else (source as PackedScene).resource_path.get_file()))


## Paper props by theme (art/blender/make_paper_props.py, rolled and scored: the meadow pilot's "C", and for the other stages' props A
## with B's stump, crate and cactus, the owner's picks 2026-10-21): the prop builders hand their places to it (`StageArt.paper_kit`).
## One kit for every stage.
const PAPER_KIT := "res://models/props/paper_props.glb"
const PAPER_KITS := {"meadow": PAPER_KIT, "grove": PAPER_KIT, "sunset": PAPER_KIT, "ocean": PAPER_KIT, "city": PAPER_KIT}


## The theme's paper props: a glTF to try (`--prop-mock=<glb>`), the shipped kit (`PAPER_KITS`), or null (none, or `--no-paper-props`).
static func paper_kit(theme_name: String) -> Node3D:
	if flag("--prop-mock=") != "":
		return _scene_from(flag("--prop-mock="))
	if OS.get_cmdline_user_args().has("--no-paper-props") or not PAPER_KITS.has(theme_name) or not ResourceLoader.exists(PAPER_KITS[theme_name]):
		return null
	return (load(PAPER_KITS[theme_name]) as PackedScene).instantiate() as Node3D


## The paper houses (art/blender/make_paper_houses_v2.py `--shape B`, the owner's pick 2026-10-21, docs/art/stages/HOUSES.md): the far
## towns' cottages, townhouses, long houses and the farm's barn, at the characters' proportions, bell and gambrel roofs. By the themes that
## have towns.
const HOUSE_KIT := "res://models/props/paper_houses.glb"
const HOUSE_THEMES := ["meadow", "ocean"]


## The theme's paper houses: a glTF to try (`--house-mock=<glb>`), the shipped kit (`HOUSE_KIT`), or null (a theme without a town, or
## `--no-houses` to compare).
static func house_kit(theme_name: String) -> Node3D:
	if flag("--house-mock=") != "":
		return _scene_from(flag("--house-mock="))
	if OS.get_cmdline_user_args().has("--no-houses") or not HOUSE_THEMES.has(theme_name) or not ResourceLoader.exists(HOUSE_KIT):
		return null
	return (load(HOUSE_KIT) as PackedScene).instantiate() as Node3D


## Each material family's print (tools/art/make_prints.py) and how strongly and how large it is laid: [print, amount, units per tile].
const PRINTS := {"grass": ["grass", 0.45, 5.0], "lawn": ["grass", 0.45, 5.0], "lawn_under": ["grass", 0.35, 5.0], "moss": ["grass", 0.35, 3.0],
	"fringe": ["grass", 0.3, 3.0], "soil": ["soil", 0.35, 4.0], "soil2": ["soil", 0.35, 4.0], "path": ["path", 0.3, 4.0],
	"log": ["wood", 0.4, 3.0], "log_end": ["wood", 0.3, 1.5], "plank": ["wood", 0.4, 3.0], "plank_b": ["wood", 0.4, 3.0],
	"plank_card": ["wood", 0.35, 3.0], "wood": ["wood", 0.4, 1.6], "wood_light": ["wood", 0.4, 1.6], "wood_dark": ["wood", 0.35, 1.6],
	"band": ["wood", 0.3, 1.2], "stem": ["grass", 0.2, 1.0], "stone": ["stone", 0.35, 2.5], "stone2": ["stone", 0.35, 2.5],
	"stone_light": ["stone", 0.35, 2.5], "straw": ["straw", 0.45, 1.4], "straw_end": ["straw", 0.4, 1.0]}
## Each stage's own families where they mean something else there (`top` is grass on the grove, sand on the canyon, boards on the
## beach, roofing on the city), and the other stages' props; over `PRINTS`.
const PRINTS_BY_THEME := {
	"grove": {"top": ["grass", 0.45, 5.0], "top_card": ["grass", 0.35, 5.0], "layer1": ["soil", 0.35, 4.0], "layer2": ["soil", 0.35, 4.0],
		"earth": ["soil", 0.35, 4.0], "rock": ["stone", 0.35, 2.5], "root": ["wood", 0.4, 3.0], "moss_shade": ["grass", 0.3, 3.0]},
	# (Sandstone on the canyon's rock, fidelity review 9, the owner's pick: `strata`'s long thin lines read as wood grain on the long beds.)
	"sunset": {"top": ["sand", 0.45, 5.0], "top_card": ["sand", 0.35, 5.0], "layer1": ["sandstone", 0.4, 4.0], "layer2": ["sandstone", 0.4, 4.0],
		"strata_a": ["sandstone", 0.4, 4.0], "strata_b": ["sandstone", 0.4, 4.0], "strata_c": ["sandstone", 0.4, 4.0]},
	"ocean": {"top": ["wood", 0.22, 3.0], "top_card": ["wood", 0.22, 3.0], "layer1": ["wood", 0.3, 3.0], "beam": ["wood", 0.4, 3.0],
		"post": ["wood", 0.4, 2.0], "iron": ["steel", 0.3, 1.5]},
	"city": {"top": ["roof", 0.4, 5.0], "top_card": ["roof", 0.3, 5.0], "layer1": ["stone", 0.3, 2.5], "steel": ["steel", 0.3, 1.5],
		"steel_card": ["steel", 0.3, 1.5], "steel_dark": ["steel", 0.3, 1.5], "steel_seam": ["steel", 0.3, 1.5], "rivet": ["steel", 0.2, 1.0],
		"steel_paint": ["steel", 0.25, 1.5], "steel_primer": ["steel", 0.3, 1.5],
		"deck_board": ["wood", 0.4, 3.0], "brick": ["stone", 0.2, 2.0], "trim": ["stone", 0.25, 2.0],
		"frame": ["wood", 0.25, 1.5], "pipe": ["steel", 0.3, 1.5], "iron": ["steel", 0.3, 1.5], "ac": ["steel", 0.3, 1.5],
		"leaf": ["grass", 0.25, 1.0]},
}
## The other stages' paper props, on every stage.
const PRINTS_PROPS := {"crate": ["wood", 0.4, 1.0], "crate_dark": ["wood", 0.35, 1.0], "bark": ["wood", 0.4, 1.0], "ring_top": ["wood", 0.3, 0.8],
	"tank": ["wood", 0.35, 1.6], "tank_dark": ["wood", 0.3, 1.6], "metal": ["steel", 0.3, 1.0], "metal_dark": ["steel", 0.3, 1.0],
	"iron": ["steel", 0.25, 1.0],
	# (The paper houses and the world-building's pieces, the quality pass after the owner's note, 2026-10-21: unprinted, they read as
	# plastic beside the printed scenery.)
	"timber": ["wood", 0.4, 1.2], "timber_dark": ["wood", 0.35, 1.2], "trim": ["wood", 0.3, 0.8], "door": ["wood", 0.4, 0.8],
	"barn": ["wood", 0.4, 1.6], "barn_door": ["wood", 0.35, 1.2], "shutter": ["wood", 0.3, 0.6], "roof": ["roof", 0.4, 2.0],
	"roof_edge": ["roof", 0.25, 1.0], "deck": ["wood", 0.4, 1.2], "hull": ["wood", 0.3, 1.6], "fish_crate": ["wood", 0.4, 0.8],
	"stall": ["wood", 0.35, 1.0], "car_a": ["wood", 0.35, 1.4], "car_b": ["wood", 0.35, 1.4], "board_frame": ["wood", 0.3, 1.2],
	"board_back": ["wood", 0.3, 1.6], "trail": ["path", 0.3, 4.0], "rock": ["stone", 0.35, 2.0], "turf": ["grass", 0.35, 2.0],
	"engine": ["steel", 0.3, 1.0], "engine_dark": ["steel", 0.25, 1.0], "steel": ["steel", 0.3, 1.5], "bus": ["steel", 0.2, 1.4],
	"gate_red": ["wood", 0.3, 1.2], "gate_dark": ["wood", 0.3, 1.2], "far_trunk": ["wood", 0.35, 2.0]}
static var _print_tex := {}


## The print for a surface of `family` on `theme_name`, or [] for none.
static func print_for(theme_name: String, family: String) -> Array:
	var fam := family.trim_suffix("_back")
	var own: Dictionary = PRINTS_BY_THEME.get(theme_name, {})
	if own.has(fam):
		return own[fam]
	# (The tops' wear prints as the top under it, fidelity review 7.)
	if fam in ["top_deep", "top_lit", "top_worn"]:
		return print_for(theme_name, "lawn" if theme_name == "meadow" else "top")
	if PRINTS.has(fam):
		return PRINTS[fam]
	return PRINTS_PROPS.get(fam, [])


## The printed surfaces (tools/art/make_prints.py, fidelity review 3's idea 2) on the modelled front and play area and the paper props, by
## each surface's material family (`PRINTS`), and the front's own paper finish level (`paper_own`; "C" the full finish, the owner's
## pick for every front, 2026-10-21): from the stage's palette ([backdrop] `paper_print`, a folder in art/stage/prints/, and `front_finish`),
## or to try, `--print-mock=<folder>` and `--front-finish=<x>`; `--no-print` to compare.
static func surfaces(root: Node3D, theme_name: String) -> void:
	var look: Dictionary = StageArt.palette(theme_name).get("backdrop", {})
	var dir := flag("--print-mock=")
	if dir == "" and String(look.get("paper_print", "")) != "":
		dir = "res://art/stage/prints/%s" % String(look.paper_print)
	if OS.get_cmdline_user_args().has("--no-print"):
		dir = ""
	var front_look := flag("--front-finish=") if flag("--front-finish=") != "" else String(look.get("front_finish", ""))
	var own := Lighting.finish_vector(front_look)
	if dir == "" and own == Vector4.ZERO:
		return
	var printed := {}
	var boosted := 0
	for mi: MeshInstance3D in root.find_children("*", "MeshInstance3D", true, false):
		var front := false
		var piece := false
		var n: Node = mi
		while n != null and n != root:
			piece = piece or n.has_meta("lit_piece") or n.has_meta("paper_prop")
			front = front or String(n.name).contains("ModelledFront")
			n = n.get_parent()
		if not piece or String(mi.name).begins_with("Mood_"):
			continue
		for s in mi.mesh.get_surface_count():
			var fam: String = mi.get_meta("family", "")
			if fam == "":
				var src := mi.mesh.surface_get_material(s)
				fam = (src.resource_name if src != null else "").get_slice(".", 0)
			var m := (mi.material_override if mi.material_override != null else mi.get_surface_override_material(s)) as ShaderMaterial
			if m == null:
				continue
			m = m.duplicate() as ShaderMaterial
			var spec := print_for(theme_name, fam)
			if dir != "" and not spec.is_empty():
				var path := dir.path_join(String(spec[0]) + ".png")
				if not _print_tex.has(path):
					# (The shipped prints are imported textures; mocks are read from their files. Either way with their own mipmaps.)
					var img: Image = null
					if path.begins_with("res://"):
						var tex := load(path) as Texture2D
						img = tex.get_image() if tex != null else null
						if img != null and img.is_compressed():
							img.decompress()
					else:
						img = Image.load_from_file(path)
					if img != null:
						img.generate_mipmaps()
					_print_tex[path] = ImageTexture.create_from_image(img) if img != null else null
				if _print_tex[path] != null:
					m.set_shader_parameter("print_tex", _print_tex[path])
					m.set_shader_parameter("print_amount", spec[1])
					m.set_shader_parameter("print_scale", spec[2])
					printed[fam] = int(printed.get(fam, 0)) + 1
			if front and own != Vector4.ZERO:
				m.set_shader_parameter("paper_own", own)
				boosted += 1
			if mi.material_override != null:
				mi.material_override = m
			else:
				mi.set_surface_override_material(s, m)
	print("stage surfaces: printed %d surfaces in %d families; %d front surfaces at the paper finish '%s'" % [
		printed.values().reduce(func(a, b): return a + b, 0), printed.size(), boosted, front_look])


## The theme's modelled tree: a glTF path to try (`--tree-mock=<glb>`, or several, comma-separated, one picked a slot), the shipped
## model or models (`TREES`, a path or a list), or null (none, or `--no-trees`).
static func tree_source(theme_name: String) -> Variant:
	var mock := tree_mock_path()
	if mock != "":
		return Array(mock.split(",")) if mock.contains(",") else mock
	if OS.get_cmdline_user_args().has("--no-trees") or not TREES.has(theme_name):
		return null
	var paths: Array = TREES[theme_name] if TREES[theme_name] is Array else [TREES[theme_name]]
	var scenes: Array = []
	for p in paths:
		if not ResourceLoader.exists(p):
			return null
		scenes.append(load(p))
	return scenes[0] if scenes.size() == 1 else scenes


static func tree_mock_path() -> String:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--tree-mock="):
			return a.substr(12)
	return ""


## Places the modelled tree (`source`: a PackedScene, or a glTF path for mocks) where each code-built tree stood (`StageArt.tree_slots`),
## at its size, painted from the stage's palette with the scenery's toon material and the model's own baked occlusion (vertex colours).
## `source` may be a list of them: each slot takes one, by its place (fidelity review 6: one tree at one size all round read as a clone).
## With the palette's [backdrop] `tree_vary` (or `--tree-vary=1`) each tree is also turned and sized a little by its place.
static func place_trees(root: Node3D, t: Dictionary, slots: Array, source: Variant) -> void:
	var sources: Array = source if source is Array else [source]
	var docs: Array = []
	for src in sources:
		var doc := GLTFDocument.new()
		var state := GLTFState.new()
		if src is String and doc.append_from_file(src, state) != OK:
			push_warning("tree mock %s could not be read" % src)
			return
		docs.append([doc, state])
	var vary := float(StageArt.palette(String(t.get("name", ""))).get("backdrop", {}).get("tree_vary", 0.0))
	if flag("--tree-vary=") != "":
		vary = float(flag("--tree-vary="))
	var grass: Color = t.get("grass", Color(0.5, 0.8, 0.3))
	# (The bark the front's roots wear, so the great tree and its roots are one tree.)
	var bark := Color(0.46, 0.37, 0.3)
	for a in OS.get_cmdline_user_args():
		if a == "--tree-bark=light":
			bark = t.get("great_bark", StageArt.palette(String(t.get("name", ""))).get("backdrop", {}).get("great_bark", Color(0.66, 0.49, 0.38)))
	var colours := {"bark": bark, "leaf_lit": Color(0.46, 0.74, 0.36), "leaf_shade": Color(0.2, 0.46, 0.3),
		"moss": grass.darkened(0.12)}
	for slot in slots:
		var at: Vector3 = slot[0]
		var key := hash(Vector2i(roundi(at.x * 10.0), roundi(at.z * 10.0)))
		# (The great tree, size 1, as it is: the front's roots run into it.)
		var great := float(slot[1]) >= 0.95
		var k := 0 if great else absi(key) % sources.size()
		var pick: Variant = sources[k]
		var tree: Node3D = (pick as PackedScene).instantiate() if pick is PackedScene else (docs[k][0] as GLTFDocument).generate_scene(docs[k][1])
		tree.name = "ModelledTree"
		tree.position = at
		tree.scale = Vector3.ONE * float(slot[1])
		if vary > 0.0 and not great:
			# (Its own turn and a size within a sixth of its slot's, from its place, so the same tree never stands twice alike.)
			tree.rotation.y = float(absi(key) % 3600) / 3600.0 * TAU * vary
			tree.scale *= 1.0 + (float(absi(key / 7) % 1000) / 1000.0 - 0.5) * 0.34 * vary
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
	if sources[0] is String:
		print("tree mock: placed %d trees from %d models" % [slots.size(), sources.size()])


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
	var fields: Array = bd.get("fields", [])
	_calm_k = float(bd.get("calm", 0.72))
	var pines: Array = bd.get("pine_greens", [Color(0.2, 0.36, 0.28)])
	# (The cliffs a step deeper, the readability pass: the grove's paper terraces right behind the fighters measured their value.)
	var out := {"grass": land, "grass_far": Color(0.38, 0.62, 0.36), "cliff": Color(0.1, 0.24, 0.23),
		"earth": Color(0.24, 0.2, 0.18), "rock": Color(0.48, 0.55, 0.44), "root": Color(0.46, 0.37, 0.3), "bark": Color(0.46, 0.37, 0.3),
		"leaf_lit": Color(0.46, 0.74, 0.36), "leaf_shade": Color(0.2, 0.46, 0.3), "moss": (t.get("grass", Color(0.5, 0.8, 0.3)) as Color).darkened(0.12),
		"fern": Color(0.36, 0.66, 0.34), "fern_shade": Color(0.18, 0.42, 0.26),
		# (The meadow's, art/blender/make_meadow_ground.py: its hill rows, the rise and its fields and hedgerows from the palette, the paper's
		# cut edge a pale card, its pines in the palette's four greens over a deep shade of them.)
		# (Calmer than the palette's lime, the owner's paper-layers pick, 2026-10-20: the hill tops and the stage's hill outshone the lawn.)
		"hill_near": _calm(bd.get("hills_near", Color(0.36, 0.72, 0.34))), "hill_far": bd.get("hills_far", Color(0.42, 0.66, 0.6)),
		"rise": _calm(bd.get("fore_band", Color(0.46, 0.76, 0.32))), "hedge": bd.get("hedgerow", Color(0.36, 0.58, 0.32)),
		"hill": _calm(Color(0.46, 0.76, 0.3)).darkened(0.06), "pine_under": (pines[0] as Color).darkened(0.5), "trunk": Color(0.32, 0.24, 0.2)}
	for k in fields.size():
		out["field_%d" % k] = _calm(fields[k])
	# (Each sheet's cut edge a pale step of its own land, catching the light: in one pale lime they read as snow, then as lime stripes.)
	# (The beach's, art/blender/make_beach_ground.py: the islands' sand and green and the headland's hills, as the code-built mounds had them.)
	out["sand"] = Color(0.98, 0.88, 0.62)
	out["isle_green"] = Color(0.3, 0.62, 0.42)
	out["head_near"] = Color(0.28, 0.52, 0.42)
	out["head_far"] = Color(0.26, 0.52, 0.52)
	# (The grove's paper far land, in the ranges' own violets.)
	# (A step deeper than the ranges' land colours: the flats take the air of their depth on top, and came out a pale lilac.)
	out["range_far"] = Color(0.62, 0.55, 0.84)
	out["range_near"] = Color(0.46, 0.4, 0.72)
	# (The canyon's paper, art/blender/make_canyon_paper.py: its strata as the modelled front has them, and the dunes' sands.)
	var soil: Color = t.get("soil", Color(0.86, 0.5, 0.32))
	# (A step deeper, the readability pass: the butte behind the fight measured the fighters' own value.)
	out["rock_a"] = soil.darkened(0.12)
	out["rock_b"] = (t.get("soil_dark", Color(0.68, 0.34, 0.24)) as Color).darkened(0.22)
	# (A step paler, not the front's 0.6: in thin plates the pale bands striped the arch like candy.)
	out["rock_c"] = soil.lerp(Color(1.0, 0.86, 0.66), 0.35).darkened(0.08)
	# (Deeper and rosier than the painted dunes' sands: as flats they take the full air of their depth and faded into the horizon.)
	out["dune_far"] = Color(0.9, 0.6, 0.5)
	# (New art toward 5, C3: the scree at the mesa's foot, a step deeper and redder than the sand.)
	out["scree"] = (t.get("soil", Color(0.86, 0.5, 0.32)) as Color).lerp(Color(0.94, 0.68, 0.4), 0.45).darkened(0.08)
	out["card_scree"] = (out["scree"] as Color).darkened(0.1)
	out["dune_near"] = Color(0.92, 0.58, 0.42)
	for fam in ["rock_a", "rock_b", "rock_c", "dune_far", "dune_near"]:
		out["card_" + fam] = (out[fam] as Color).darkened(0.1)
	for fam in ["range_far", "range_near", "cliff", "grass", "grass_far"]:
		var pale := 0.04 if fam == "cliff" else 0.18
		out["card_" + fam] = (out[fam] as Color).darkened(0.12) if fam.begins_with("range") else (out[fam] as Color).lerp(Color(0.92, 0.93, 0.86), pale)
	for fam in ["sand", "isle_green", "head_near", "head_far"]:
		# (The sand's edge a pale step; the green land's a shade under its top: far off, the camera sees the cut edges more than the tops,
		# and in a pale step the headland read as a mint wedding cake.)
		out["card_" + fam] = (out[fam] as Color).lerp(Color(0.96, 0.95, 0.88), 0.2) if fam == "sand" else (out[fam] as Color).darkened(0.12)
	for band in ["far", "near", "rise", "hill"]:
		var own: Color = out["hill_" + band] if band in ["far", "near"] else out[band]
		out["card_" + band] = own.lerp(Color(0.92, 0.93, 0.86), 0.22)
	for k in pines.size():
		out["pine_%d" % k] = pines[k]
	# (The meadow's patchwork land, docs/art/stages/BACKGROUNDS.md: its lanes a pale warm sheet, the soft shade things cast on the fields,
	# stones and flowers dotted over them.)
	out["lane"] = Color(0.95, 0.91, 0.74)
	out["card_lane"] = Color(0.84, 0.78, 0.6)
	# (Fidelity review 11: the cream lanes were the brightest thing in the meadow's lower third, white ribbons down the terraces. The
	# palette's [backdrop] `lane_look` or `--lane=A|B|C`: A the farm track's dust, B a deeper trodden earth, C the farmyard's own earth.)
	var lane_look := String(bd.get("lane_look", ""))
	for arg in OS.get_cmdline_user_args():
		if arg.begins_with("--lane="):
			lane_look = arg.substr(7)
	if lane_look == "A":
		out["lane"] = Color(0.84, 0.74, 0.54)
		out["card_lane"] = Color(0.7, 0.6, 0.44)
	elif lane_look == "B":
		out["lane"] = Color(0.74, 0.62, 0.44)
		out["card_lane"] = Color(0.6, 0.5, 0.36)
	elif lane_look == "C":
		out["lane"] = Color(0.62, 0.5, 0.34)
		out["card_lane"] = Color(0.5, 0.4, 0.28)
	out["shade"] = (out["rise"] as Color).darkened(0.32)
	out["stone"] = Color(0.82, 0.81, 0.78)
	out["flower_a"] = Color(0.98, 0.64, 0.72)
	out["flower_b"] = Color(1.0, 0.86, 0.42)
	# (The meadow's tidy quilt, after the owner's note that PB read muddy: clean greens after the reference's lime, a warm wheat, the cut
	# edges a deeper green instead of cream, hedgerows a dark leaf green.)
	out["qf_a"] = Color(0.46, 0.78, 0.3)
	out["qf_b"] = Color(0.37, 0.68, 0.27)
	out["qf_c"] = Color(0.55, 0.83, 0.36)
	out["wheat"] = Color(0.92, 0.82, 0.44)
	out["qcard"] = Color(0.27, 0.5, 0.22)
	out["qhedge"] = Color(0.22, 0.43, 0.22)
	out["qgap"] = Color(0.28, 0.54, 0.24)
	# (The meadow's hedged farmland, 2026-10-22: rows in the wheat and ploughed fields, young crops in green rows on brown, the lanes'
	# grass crown, the terraces' banks in a deep grass green: in earth brown they read muddy.)
	out["wheat_row"] = Color(0.8, 0.66, 0.32)
	# (New art toward 5, M1: the hill's terraced fields held by walls of the stage wall's own warm fieldstone.)
	out["terrace_wall"] = Color(0.66, 0.58, 0.46)
	out["terrace_wall_b"] = Color(0.56, 0.5, 0.42)
	# (New art toward 5, M2: the meadow's stream down the terraces, its banks, its footbridge's planks, the wet stones along it.)
	out["stream"] = Color(0.44, 0.68, 0.84)
	out["card_stream"] = Color(0.32, 0.46, 0.4)
	out["plank"] = Color(0.66, 0.5, 0.34)
	out["card_plank"] = Color(0.48, 0.36, 0.26)
	out["wet_stone"] = Color(0.5, 0.52, 0.5)
	out["plough"] = Color(0.6, 0.44, 0.32)
	out["plough_row"] = Color(0.48, 0.34, 0.25)
	out["crop_row"] = Color(0.5, 0.74, 0.3)
	out["verge"] = Color(0.5, 0.74, 0.34)
	out["bank"] = Color(0.27, 0.44, 0.27)
	for fam in ["hill_near", "hill_far"]:
		out[fam + "_l"] = (out[fam] as Color).lightened(0.1)
		out[fam + "_d"] = (out[fam] as Color).darkened(0.08)
	# (The meadow's shelves after its reference, the papercraft camp: bright lime grass, every other shelf a step deeper, a crisp dark
	# green-grey ledge under each, its grass lip a little deeper than the top.)
	out["shelf"] = Color(0.44, 0.76, 0.3)
	out["shelf_b"] = Color(0.38, 0.69, 0.29)
	out["shelf_lip"] = Color(0.32, 0.6, 0.26)
	out["ledge"] = Color(0.2, 0.31, 0.24)
	# (The grove's floating ledges: the violet mist under them, tufts on the tops, the far forest's canopy.)
	out["mist"] = Color(0.8, 0.76, 0.93)
	out["card_mist"] = Color(0.68, 0.63, 0.85)
	out["tuft"] = (out["grass"] as Color).lightened(0.12)
	out["canopy"] = Color(0.4, 0.4, 0.66)
	out["isle_rock"] = Color(0.34, 0.31, 0.4)
	# (The grove's hero ledges: the grass lip a deeper green than the top, the vines darker still, the abyss a deep violet.)
	out["lip"] = (out["grass"] as Color).darkened(0.18)
	out["vine"] = Color(0.16, 0.34, 0.24)
	out["abyss"] = Color(0.26, 0.2, 0.42)
	out["abyss_far"] = Color(0.4, 0.34, 0.62)
	out["cliff_b"] = Color(0.16, 0.32, 0.32)
	# (Fidelity review 6, the grove's rock shaped: each block's bevel a lit rim in a lighter moss-grey; and the abyss a misty valley, its
	# mist a step lighter than the abyss with a paler cut edge.)
	out["cliff_lit"] = Color(0.42, 0.56, 0.48)
	out["abyss_mist"] = Color(0.42, 0.36, 0.62)
	out["card_abyss_mist"] = Color(0.56, 0.5, 0.78)
	out["valley_lit"] = Color(0.4, 0.5, 0.62)
	out["valley_shade"] = Color(0.28, 0.3, 0.5)
	out["valley_bark"] = Color(0.32, 0.28, 0.44)
	# (G2: the deep valley's floor far below the islands and its river, in the valley's own hazed violets.)
	out["valley_floor"] = Color(0.26, 0.28, 0.44)
	out["bank_mist"] = Color(0.8, 0.76, 0.92)
	out["bank_mist_low"] = Color(0.64, 0.6, 0.84)
	out["bank_mist_top"] = Color(0.93, 0.91, 0.99)
	out["valley_river"] = Color(0.62, 0.68, 0.9)
	# (New art toward 5, G1: the grove's falls in two pale blues, their spray, and the mist's farther layers paler with depth.)
	out["fall"] = Color(0.84, 0.93, 0.98)
	out["fall_shade"] = Color(0.6, 0.76, 0.9)
	out["spray"] = Color(0.92, 0.94, 1.0)
	out["abyss_mist2"] = Color(0.5, 0.44, 0.7)
	out["abyss_mist3"] = Color(0.58, 0.52, 0.78)
	# (One rock for the grove, fidelity review 9: the island's underside is grey-green stone, the terraces' blocks a deep teal, two
	# languages side by side. The palette's [backdrop] `cliff_stone` (or `--cliff-stone=`), how far the terraces' rock turns to the
	# island's stone, its value kept deep for the fighters in front of it.)
	var stone_k := float(bd.get("cliff_stone", 0.0))
	for arg in OS.get_cmdline_user_args():
		if arg.begins_with("--cliff-stone="):
			stone_k = float(arg.substr(14))
	if stone_k > 0.0:
		var stone: Color = out["rock"]
		out["cliff"] = (out["cliff"] as Color).lerp(stone.darkened(0.55), stone_k)
		out["cliff_b"] = (out["cliff_b"] as Color).lerp(stone.darkened(0.42), stone_k)
		out["cliff_lit"] = (out["cliff_lit"] as Color).lerp(stone.darkened(0.08), stone_k)
		out["card_cliff"] = (out["cliff"] as Color).lerp(Color(0.92, 0.93, 0.86), 0.04)
	# (The canyon's floor in paper: its sand sheets in three tones round the desert floor's, their cut edges, wind ripples, the dry wash
	# and its banks, pebbles, scrub and the shade under it.)
	var sand: Color = bd.get("desert_floor", Color(0.94, 0.68, 0.4))
	out["sand_a"] = sand.lerp(Color(1.0, 0.9, 0.7), 0.25)
	out["sand_b"] = sand.darkened(0.08)
	out["sand_c"] = sand.lerp(Color(0.92, 0.62, 0.56), 0.3)
	for fam in ["sand_a", "sand_b", "sand_c"]:
		out["card_" + fam] = (out[fam] as Color).darkened(0.16)
	out["ripple"] = sand.lerp(Color(1.0, 0.95, 0.84), 0.28)
	out["wash"] = sand.lerp(Color(0.96, 0.9, 0.82), 0.55)
	out["card_wash"] = sand.darkened(0.2)
	out["pebble"] = sand.lerp(Color(0.62, 0.4, 0.36), 0.5)
	out["scrub"] = Color(0.56, 0.62, 0.38)
	out["sand_shade"] = sand.darkened(0.22)
	out["isle_moss"] = Color.from_hsv((out["grass"] as Color).h, (out["grass"] as Color).s * 0.6, (out["grass"] as Color).v * 0.72)
	return out


## A palette green a step calmer: less saturated and a touch darker.
static func _calm(c: Color) -> Color:
	return Color.from_hsv(c.h, minf(c.s * _calm_k, 1.0), c.v * 0.95)


## How much `_calm` keeps of a colour's saturation, the palette's backdrop `calm` (0.72 unless it says).
static var _calm_k := 0.72


## Paints a modelled background piece: each family's toon material, its own baked occlusion (vertex colours), and the air at its depth.
static func _paint_ground(node: Node3D, colours: Dictionary, t := {}) -> void:
	# (The ground's own finish, fidelity review 10: the meadow's hill under the wall a plain faceted green at the wide camera. The
	# palette's [backdrop] `ground_finish` or `--ground-finish=A|B`: painted variation on the land's families, A broad (`GROUND_DRIFT`),
	# B finer and stronger, patches a few units across.)
	var finish := String((StageArt.palette(String(t.get("name", ""))).get("backdrop", {}) as Dictionary).get("ground_finish", ""))
	if flag("--ground-finish=") != "":
		finish = flag("--ground-finish=").to_upper()
	# (The beach's headland painted, fidelity review 15 item 4: through the air at about 200 its folded hill and pop-up flats were one pale
	# sage card. `--headland=A|B`, see HEADLAND_TRY.)
	var head: Dictionary = HEADLAND_TRY.get(flag("--headland=").to_upper(), {}) if String(t.get("name", "")) == "ocean" else {}
	for mi: MeshInstance3D in node.find_children("*", "MeshInstance3D", true, false):
		for s in mi.mesh.get_surface_count():
			var src := mi.mesh.surface_get_material(s)
			var fam := src.resource_name if src != null else "grass"
			var colour: Color = (head.get("colours", {}) as Dictionary).get(fam, colours.get(fam, Color(0.6, 0.6, 0.6)))
			var m := (FighterView.toon(colour, false) as ShaderMaterial).duplicate() as ShaderMaterial
			m.set_shader_parameter("facets", fam != "bark")
			m.set_shader_parameter("vertex_shade", true)
			m.set_shader_parameter("ramp_softness", 0.12)
			if finish in ["A", "B"] and fam in GROUND_DRIFT:
				var d: Vector4 = GROUND_DRIFT[fam]
				m.set_shader_parameter("paint_drift", d if finish == "A" else Vector4(d.x * 1.5, d.y * 1.3, d.z * 0.4, d.w))
			if fam in ["head_near", "head_far"] and head.has("drift"):
				m.set_shader_parameter("paint_drift", head.drift)
				m.set_shader_parameter("ramp_center", float(head.get("ramp_center", 0.45)))
			mi.set_surface_override_material(s, m)
		mi.set_meta("prebaked", true)
		var depth := -(mi.global_transform * mi.mesh.get_aabb().get_center()).z
		if head.has("air") and String(mi.name).begins_with("head"):
			var amount := StageArt._air_amount(depth) * float(head.air)
			StageArt._mute(mi, amount.x, (StageArt._palette.get("air", {}) as Dictionary).get("haze", Color(0.8, 0.85, 0.95)), amount.y, false)
			continue
		StageArt._air(mi, depth)


## A front's strata lifted toward a pale warm sand by the theme's `front_lift` (`--front-lift=` to try): fidelity review 5, the canyon's
## front dark and saturated (value 0.30, saturation 0.61) weighing down the lower half of the frame.
## The beach's headland painted, on trial (fidelity review 15 item 4, `--headland=A|B`), as the meadow's peaks have it:
##   A  painted patches over its folded hill and pop-up flats, the tops warmer and the walls cooler (toon_common `paint_drift`);
##   B  A, its greens a step deeper and richer, two fifths of the air over it, and the toon ramp's turn moved up (`ramp_center` 0.62) so
##      the folds turned from the sun fall into shade: a lit side and a shade side toward the town.
## (A first try at patches 2.0 and half the air barely showed at its distance.)
const HEADLAND_TRY := {
	"A": {"drift": Vector4(3.2, 1.6, 10.0, 1.2)},
	"B": {"drift": Vector4(3.2, 1.6, 10.0, 1.2), "air": 0.4, "ramp_center": 0.62,
		"colours": {"head_near": Color(0.22, 0.5, 0.36), "head_far": Color(0.22, 0.46, 0.5)}},
}


## The painted variation on a ground's land families (`ground_finish`): broader than a play area's, the size a hillside's.
## The meadow's hill, the grove's abyss and its mist, the canyon's floor.
const GROUND_DRIFT := {"hill": Vector4(1.6, 0.9, 22.0, 0.0), "bank": Vector4(1.3, 0.7, 18.0, 0.0), "hill_near": Vector4(1.3, 0.8, 30.0, 0.0),
	"rise": Vector4(1.3, 0.8, 26.0, 0.0), "abyss": Vector4(1.8, 1.0, 16.0, 0.0), "abyss_far": Vector4(1.4, 0.8, 24.0, 0.0),
	"abyss_mist": Vector4(1.6, 1.0, 14.0, 0.0), "sand_a": Vector4(1.3, 0.8, 20.0, 0.0), "sand_b": Vector4(1.3, 0.8, 20.0, 0.0),
	"sand_c": Vector4(1.3, 0.8, 20.0, 0.0)}


static func _front_lift(t: Dictionary, c: Color) -> Color:
	var lift := float(t.get("front_lift", 0.0))
	if flag("--front-lift=") != "":
		lift = float(flag("--front-lift="))
	return c.lerp(Color(0.96, 0.84, 0.72), lift)


## The modelled ground and bush for this exact stage: glTF paths to try (`--ground-mock=`, `--bush-mock=`), the shipped models
## (`GROUNDS`, as PackedScenes), or [] (none, or `--no-ground`).
static func ground_source(bake_key: String) -> Array:
	if flag("--ground-mock=") != "":
		return [flag("--ground-mock="), flag("--bush-mock=")]
	if OS.get_cmdline_user_args().has("--no-ground") or not GROUNDS.has(bake_key):
		return []
	var paths: Array = GROUNDS[bake_key]
	if not ResourceLoader.exists(paths[0]) or (paths[1] != "" and not ResourceLoader.exists(paths[1])):
		return []
	return [load(paths[0]), load(paths[1]) if paths[1] != "" else null]


## The height of the highest surface of `node`'s meshes straight under (`x`, `z`), or NAN where there is none.
static func _height_at(node: Node3D, x: float, z: float) -> float:
	var best := NAN
	var from := Vector3(x, 200.0, z)
	for mi: MeshInstance3D in node.find_children("*", "MeshInstance3D", true, false):
		# (Its transform within `node`, which stands at the origin: the scene may not be in the tree yet.)
		var xf := Transform3D.IDENTITY
		var n: Node = mi
		while n != node and n is Node3D:
			xf = (n as Node3D).transform * xf
			n = n.get_parent()
		var faces := mi.mesh.get_faces()
		for i in range(0, faces.size(), 3):
			var hit = Geometry3D.ray_intersects_triangle(from, Vector3.DOWN, xf * faces[i], xf * faces[i + 1], xf * faces[i + 2])
			if hit != null and (is_nan(best) or (hit as Vector3).y > best):
				best = (hit as Vector3).y
	return best


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
		ground.set_meta("lit_piece", true)
		root.add_child(ground)
		_paint_ground(ground, colours, t)
		# (Trees the ground asks for, `treeslot` empties: the theme's modelled tree stands at each, as at the code-built trees' places.)
		for n in ground.find_children("treeslot*", "", true, false):
			StageArt.tree_slots.append([(n as Node3D).position, (n as Node3D).scale.x])
			n.get_parent().remove_child(n)
			n.free()
		# (A floor scattered in paper stands in for the code-built scrub and stones on it.)
		if ground.find_children("scatter*", "", true, false).size() > 0 and root.get_parent() != null:
			for n in root.get_parent().find_children("*", "GeometryInstance3D", true, false):
				if n.has_meta("floor_scatter"):
					(n as Node3D).visible = false
		# (A modelled far land stands in for the painted ridges.)
		if ground.find_children("range*", "", true, false).size() > 0:
			for r in root.find_children("*", "GeometryInstance3D", true, false):
				if r.has_meta("ridge"):
					(r as Node3D).visible = false
		# (The meadow's farm stands on the modelled valley, not on the code-built folds it was seated on.)
		var farm := root.get_node_or_null("Farm") as Node3D
		if farm != null:
			var y := _height_at(ground, farm.position.x, farm.position.z)
			if not is_nan(y):
				farm.position.y = y
		# (And the hay, barrels and rocks on the stage's hill, each by how far the model's surface lies from the slope it stood on.)
		var props := root.get_node_or_null("HillProps")
		if props != null:
			for c in props.get_children():
				var n := c as Node3D
				var y := _height_at(ground, n.position.x, n.position.z)
				if not is_nan(y) and n.has_meta("seat"):
					n.position.y += y - float(n.get_meta("seat"))
	for slot in StageArt.bush_slots:
		var bush := _scene_from(source[1])
		if bush == null:
			break
		bush.position = slot[0]
		bush.scale = Vector3.ONE * float(slot[1])
		bush.rotation_degrees.y = float(slot[2])
		root.add_child(bush)
		_paint_ground(bush, colours)


## The front's roots (fidelity review 13 item 5: under the grove's pink light the bark brown read pink, ribbons not roots): the palette's
## [backdrop] `root_colour` or `--roots=A|B`, a deeper bark; the brown as before without.
static func _root_colour(t: Dictionary) -> Color:
	var c: Color = StageArt.palette(String(t.get("name", ""))).get("backdrop", {}).get("root_colour", Color(0.46, 0.37, 0.3))
	var f := flag("--roots=")
	if f != "":
		c = {"A": Color(0.38, 0.26, 0.17), "B": Color(0.3, 0.2, 0.13)}.get(f.to_upper(), c)
	return c

