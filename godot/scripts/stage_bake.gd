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

const DIR := "res://art/baked/%s/"
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
static func _bakeable(mi: MeshInstance3D) -> bool:
	if mi.mesh == null or not mi.is_visible_in_tree() or mi.mesh.get_surface_count() == 0:
		return false
	for s in mi.mesh.get_surface_count():
		if not BAKEABLE.has(_shader_path(mi.get_active_material(s))):
			return false
	return true


static func _occludes(gi: GeometryInstance3D) -> bool:
	if not gi.is_visible_in_tree():
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
static func export_scene(root: Node3D, bake_key: String) -> int:
	var dir := DIR % bake_key
	DirAccess.make_dir_recursive_absolute(ProjectSettings.globalize_path(dir))
	# (Godot's importer leaves the folder alone: the files are read at run time as they are.)
	FileAccess.open(dir + ".gdignore", FileAccess.WRITE).store_string("")
	var out := Node3D.new()
	out.name = "Bake"
	var manifest := {"key": bake_key, "meshes": []}
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
