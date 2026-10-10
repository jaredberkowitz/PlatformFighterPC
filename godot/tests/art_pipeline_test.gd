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


func _initialize() -> void:
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
	print("art pipeline test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)
