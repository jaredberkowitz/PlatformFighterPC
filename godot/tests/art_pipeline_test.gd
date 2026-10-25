extends SceneTree
## The game's end of the art pipeline (docs/ART_WORKFLOW.md): an imported model's plain materials become cel materials of the same
## colour, a missing prop or backdrop is simply absent, and painted backdrop layers are placed from their list.
## Run: Godot --headless --path godot --script res://tests/art_pipeline_test.gd

const Toonify := preload("res://scripts/toonify.gd")
const StageArt := preload("res://scripts/stage_art.gd")
const FighterView := preload("res://scripts/fighter_view.gd")

var failed := false


func check(ok: bool, what: String) -> void:
	if not ok:
		print("FAIL ", what)
		failed = true


## Every SVG under res://art is read as a file (svg_art.gd) and so must be kept, not imported as a texture: an imported one is left
## out of the exported game, and that face or print goes missing in the build only.
func _check_svgs_kept(dir: String) -> void:
	for sub in DirAccess.get_directories_at(dir):
		_check_svgs_kept(dir.path_join(sub))
	for file in DirAccess.get_files_at(dir):
		if file.ends_with(".svg.import"):
			var text := FileAccess.get_file_as_string(dir.path_join(file))
			check(text.contains("importer=\"keep\""), "%s is kept as a file (importer=\"keep\")" % dir.path_join(file))


func _initialize() -> void:
	_check_svgs_kept("res://art")
	# A mesh with a plain material, as the import script exports them.
	var mi := MeshInstance3D.new()
	var box := BoxMesh.new()
	var plain := StandardMaterial3D.new()
	plain.albedo_color = Color(0.8, 0.2, 0.1)
	box.material = plain
	mi.mesh = box
	var holder := Node3D.new()
	holder.add_child(mi)
	check(Toonify.apply(holder) == 1, "one surface swapped")
	var m := mi.get_surface_override_material(0) as ShaderMaterial
	check(m != null and m.shader == FighterView.TOON_SHADER, "it is the cel material")
	check(m != null and (m.get_shader_parameter("albedo") as Color).is_equal_approx(Color(0.8, 0.2, 0.1)), "in the same colour")
	check(m != null and m.next_pass != null, "with the ink outline")
	# A palette texture is kept on the cel material.
	var tex := ImageTexture.create_from_image(Image.create_empty(4, 4, false, Image.FORMAT_RGB8))
	plain.albedo_texture = tex
	Toonify.apply(holder)
	m = mi.get_surface_override_material(0) as ShaderMaterial
	check(m.get_shader_parameter("use_tex") == true and m.get_shader_parameter("albedo_tex") == tex, "the texture is kept")
	check(Toonify.load_prop("no_such_prop") == null, "a missing prop is null")
	# No painted layers for a theme that has none: nothing placed, no errors.
	var root3 := Node3D.new()
	check(StageArt.painted_layers(root3, "no_such_theme") == 0, "no layers, nothing placed")
	holder.free()
	root3.free()
	# The base body (art/models/base_body, built by make_rigged_blob.py --body): the same skeleton and clips, its face shell and fists.
	FighterView.use_base_body = true
	var view := FighterView.new()
	view.build(0)
	check(view.anim != null and view.anim.has_animation("idle") and view.anim.has_animation("sword_fsmash"), "the base body plays the clips")
	check(view.face_mesh != null, "it has the face shell")
	check(view.skeleton != null and view.skeleton.find_bone("hand.R") >= 0, "on the game's skeleton")
	check(view.shoulder.y > 1.0 and view.arm_reach > 0.3, "its shoulder and reach come from base_rig.json")
	var parts := []
	for n in view.find_children("*", "MeshInstance3D", true, false):
		parts.append(str(n.name))
	check(parts.has("Skin") and (parts.has("Hand_L") or parts.has("Hand.L")), "skin and glove fists: %s" % [parts.slice(0, 6)])
	view.free()
	FighterView.use_base_body = false
	FighterView.release_caches()
	_check_palettes()
	print("art pipeline test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)


## Every stage's palette file (res://stages/palettes, docs/ART_WORKFLOW.md) loads with the theme keys the stage art reads and an [air]
## curve; and building each stage's scenery asks for no colour its palette lacks.
func _check_palettes() -> void:
	for name in StageArt.THEME_NAMES:
		var pal := StageArt.palette(name)
		var theme: Dictionary = pal.get("theme", {})
		for key in ["sky_top", "sky_horizon", "soil", "soil_dark", "grass", "grass_light", "wood", "wood_dark", "pattern", "cap", "platform"]:
			check(theme.has(key), "%s palette has theme %s" % [name, key])
		var air: Dictionary = pal.get("air", {})
		for key in ["start", "end", "desat", "lift", "haze"]:
			check(air.has(key), "%s palette has air %s" % [name, key])
		check(pal.get("mist", {}).has("layers"), "%s palette has mist layers" % name)
		var t := StageArt.theme({"backdrop": name})
		check(t.name == name, "%s theme loads by name" % name)
		var root := Node3D.new()
		StageArt.palette_misses.clear()
		StageArt.backdrop(root, t, [-11.0, 11.0, 0.0, -6.0])
		check(StageArt.palette_misses.is_empty(), "%s scenery finds all its colours: missing %s" % [name, StageArt.palette_misses])
		if name == "city":
			# The modelled near skyline (art/blender/make_city_towers.py) was built from the layout the city lays out now.
			check(not root.find_children("skyline_near_*", "MeshInstance3D", true, false).is_empty(),
				"the modelled city skyline matches the city's layout: re-export and rebuild it (make_city_towers.py)")
		root.free()
