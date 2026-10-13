extends SceneTree
## The base body's head straight on, framed like the concept sheet's front view (art/concept/base_body_turnaround.jpg), for comparing the
## drawn face with the concept: one cell per default look, or `--look=N`. Needs a window (not --headless).
## Run: Godot --path godot --script res://tests/face_sheet.gd -- --out=<file.png> [--look=N] [--expr=<expression>] [--turn=<degrees>]

const FighterView := preload("res://scripts/fighter_view.gd")
const Loadout := preload("res://scripts/loadout.gd")
const Lighting := preload("res://scripts/lighting.gd")
const FaceLook := preload("res://scripts/face_look.gd")

const CELL := Vector2i(420, 420)


func _initialize() -> void:
	var out := "user://face_sheet.png"
	var looks := [0, 1, 2, 3]
	var expr := ""
	var turn := 0.0
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--out="):
			out = a.substr(6)
		elif a.begins_with("--look="):
			looks = [int(a.substr(7))]
		elif a.begins_with("--expr="):
			expr = a.substr(7)
		elif a.begins_with("--turn="):
			turn = float(a.substr(7))
	var vp := SubViewport.new()
	vp.size = Vector2i(CELL.x * looks.size(), CELL.y)
	vp.own_world_3d = true
	vp.msaa_3d = Viewport.MSAA_4X
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	root.add_child(vp)
	var world := Node3D.new()
	vp.add_child(world)
	var env := Environment.new()
	env.background_mode = Environment.BG_COLOR
	env.background_color = Color(0.75, 0.75, 0.76)
	var we := WorldEnvironment.new()
	we.environment = env
	world.add_child(we)
	Lighting.apply(env, world, true)
	var info: Dictionary = FighterView._base_rig_info()
	# (Framed on the face drawing's middle, which is the round head's middle.)
	var head_y := float((info.get("face", [1.8, 0.8]) as Array)[0])
	var head_r := float((info.get("head_radii", [0.47, 0.4, 0.45]) as Array)[0])
	var cam := Camera3D.new()
	cam.projection = Camera3D.PROJECTION_ORTHOGONAL
	# (The concept's crop: the head's width is about 0.96 of the cell.)
	cam.size = head_r * 2.0 / 0.96
	cam.keep_aspect = Camera3D.KEEP_HEIGHT
	world.add_child(cam)
	var span := cam.size * float(looks.size())
	cam.look_at_from_position(Vector3(0, head_y, 10.0), Vector3(0, head_y, 0))
	for k in looks.size():
		var v := FighterView.new()
		world.add_child(v)
		v.build(looks[k], Loadout.default_for(looks[k]))
		v.blade_pivot.visible = false
		v.percent_label.visible = false
		v.name_label.visible = false
		v.position = Vector3(-span * 0.5 + span * (k + 0.5) / looks.size(), 0.0, 0.0)
		v.model.rotation.y = deg_to_rad(turn)
		if expr != "":
			v.set_expression(FaceLook.EXPRESSIONS.get(expr, {}).merged({"name": expr}))
	await create_timer(0.8).timeout
	var img := vp.get_texture().get_image()
	img.save_png(out)
	quit(0)
